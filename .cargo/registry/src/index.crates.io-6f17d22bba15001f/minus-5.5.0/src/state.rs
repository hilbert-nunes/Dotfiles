//! Contains types that hold run-time information of the pager.

#[cfg(feature = "search")]
use crate::search::{SearchMode, SearchOpts};

use crate::{
    error::{MinusError, TermError},
    input::{self, HashedEventRegister},
    minus_core::{
        self,
        utils::text::{self, AppendStyle},
    },
    screen::Screen,
    ExitStrategy, LineNumbers,
};
use crossterm::{terminal, tty::IsTty};
#[cfg(feature = "search")]
use parking_lot::Condvar;
use parking_lot::Mutex;
#[cfg(feature = "search")]
use std::collections::BTreeSet;
use std::{
    collections::hash_map::RandomState,
    collections::HashMap,
    convert::TryInto,
    io::stdout,
    io::Stdout,
    sync::{atomic::AtomicBool, Arc},
};

use crate::minus_core::{commands::Command, ev_handler::handle_event};
use crossbeam_channel::Receiver;

#[cfg(feature = "search")]
#[cfg_attr(docsrs, doc(cfg(feature = "search")))]
#[allow(clippy::module_name_repetitions)]
/// Contains information about the current search
pub struct SearchState {
    /// Direction of search
    ///
    /// See [`SearchMode`] for available options
    pub search_mode: SearchMode,
    /// Stores the most recent search term
    pub(crate) search_term: Option<regex::Regex>,
    /// Lines where searches have a match
    /// In order to avoid duplicate entries of lines, we keep it in a [`BTreeSet`]
    pub(crate) search_idx: BTreeSet<usize>,
    /// Index of search item currently in focus
    /// It should be 0 even when no search is in action
    pub(crate) search_mark: usize,
    /// Function to run before running an incremental search.
    ///
    /// If the function returns a `false`, the incremental search is cancelled.
    pub(crate) incremental_search_condition:
        Box<dyn Fn(&SearchOpts) -> bool + Send + Sync + 'static>,
}

#[cfg(feature = "search")]
impl Default for SearchState {
    fn default() -> Self {
        let incremental_search_condition = Box::new(|so: &SearchOpts| {
            so.string.len() > 1
                && so
                    .incremental_search_options
                    .as_ref()
                    .unwrap()
                    .initial_formatted_lines
                    .len()
                    <= 5000
        });
        Self {
            search_mode: SearchMode::Unknown,
            search_term: None,
            search_idx: BTreeSet::new(),
            search_mark: 0,
            incremental_search_condition,
        }
    }
}

/// Holds all information and configuration about the pager during
/// its run time.
///
/// This type is exposed so that end-applications can implement the
/// [`InputClassifier`](input::InputClassifier) trait which requires the `PagerState` to be passed
/// as a parameter
///
/// Various fields are made public so that their values can be accessed while implementing the
/// trait.
#[allow(clippy::module_name_repetitions)]
pub struct PagerState {
    /// Configuration for line numbers. See [`LineNumbers`]
    pub line_numbers: LineNumbers,
    /// Any message to display to the user at the prompt
    /// The first element contains the actual message, while the second element tells
    /// whether the message has changed since the last display.
    pub message: Option<String>,
    /// The upper bound of scrolling.
    ///
    /// This is useful for keeping track of the range of lines which are currently being displayed on
    /// the terminal.
    /// When `rows - 1` is added to the `upper_mark`, it gives the lower bound of scroll.
    ///
    /// For example if there are 10 rows is a terminal and the data to display has 50 lines in it/
    /// If the `upper_mark` is 15, then the first row of the terminal is the 16th line of the data
    /// and last row is the 24th line of the data.
    pub upper_mark: usize,
    /// Direction of search
    ///
    /// See [`SearchMode`] for available options
    ///
    /// **WARNING: This item has been deprecated in favour of [SearchState::search_mode] availlable
    /// by the [PagerState::search_state] field. Any new code should prefer using it instead of this one.**
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, cfg(feature = "search"))]
    pub search_mode: SearchMode,
    /// Available rows in the terminal
    pub rows: usize,
    /// Available columns in the terminal
    pub cols: usize,
    /// This variable helps in scrolling more than one line at a time
    /// It keeps track of all the numbers that have been entered by the user
    /// until any of `j`, `k`, `G`, `Up` or `Down` is pressed
    pub prefix_num: String,
    /// Describes whether minus is running and in which mode
    pub running: &'static Mutex<crate::RunMode>,
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, cfg(feature = "search"))]
    pub search_state: SearchState,
    pub screen: Screen,
    /// Unterminated lines
    /// Keeps track of the number of lines at the last of [PagerState::formatted_lines] which are
    /// not terminated by a newline
    pub(crate) unterminated: usize,
    /// The prompt displayed at the bottom wrapped to available terminal width
    pub(crate) prompt: String,
    /// The input classifier to be called when a input is detected
    pub(crate) input_classifier: Box<dyn input::InputClassifier + Sync + Send>,
    /// Functions to run when the pager quits
    pub(crate) exit_callbacks: Vec<Box<dyn FnMut() + Send + Sync + 'static>>,
    /// The behaviour to do when user quits the program using `q` or `Ctrl+C`
    /// See [`ExitStrategy`] for available options
    pub(crate) exit_strategy: ExitStrategy,
    /// The prompt that should be displayed to the user, formatted with the
    /// current search index and number of matches (if the search feature is enabled),
    /// and the current numbers inputted to scroll
    pub(crate) displayed_prompt: String,
    /// Whether to show the prompt on the screen
    pub(crate) show_prompt: bool,
    /// Do we want to page if there is no overflow
    #[cfg(feature = "static_output")]
    pub(crate) run_no_overflow: bool,
    /// A `HashMap` that describes where first row of each line in [`PagerState::lines`] in placed
    /// inside [`PagerState::formatted_lines`].
    /// This is helpful when we defining keybindings like `[n]G` where `[n]` denotes which line to jump to.
    /// See [`input::generate_default_bindings`] for exact definition on how it is implemented.
    pub(crate) lines_to_row_map: HashMap<usize, usize>,
}

impl PagerState {
    pub(crate) fn new() -> Result<Self, TermError> {
        let (rows, cols);

        if cfg!(test) {
            // In tests, set  number of columns to 80 and rows to 10
            cols = 80;
            rows = 10;
        } else if stdout().is_tty() {
            // If a proper terminal is present, get size and set it
            let size = terminal::size()?;
            cols = size.0 as usize;
            rows = size.1 as usize;
        } else {
            // For other cases beyond control
            cols = 1;
            rows = 1;
        };

        let prompt = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("minus"))
            .file_name()
            .map_or_else(
                || std::ffi::OsString::from("minus"),
                std::ffi::OsStr::to_os_string,
            )
            .into_string()
            .unwrap_or_else(|_| String::from("minus"));

        let mut state = Self {
            line_numbers: LineNumbers::Disabled,
            upper_mark: 0,
            unterminated: 0,
            prompt,
            running: &minus_core::RUNMODE,
            exit_strategy: ExitStrategy::ProcessQuit,
            input_classifier: Box::<HashedEventRegister<RandomState>>::default(),
            exit_callbacks: Vec::with_capacity(5),
            message: None,
            screen: Screen::default(),
            displayed_prompt: String::new(),
            show_prompt: true,
            #[cfg(feature = "static_output")]
            run_no_overflow: false,
            #[cfg(feature = "search")]
            search_mode: SearchMode::default(),
            #[cfg(feature = "search")]
            search_state: SearchState::default(),
            // Just to be safe in tests, keep at 1x1 size
            cols,
            rows,
            prefix_num: String::new(),
            lines_to_row_map: HashMap::new(),
        };

        state.format_prompt();
        Ok(state)
    }

    /// Generate the initial [`PagerState`]
    ///
    /// [`init_core`](crate::minus_core::init::init_core) calls this functions for creating the PagerState.
    ///
    /// This function creates a default [`PagerState`] and fetches all events present in the receiver
    /// to create the initial state. This is done before starting the pager so that
    /// the optimizationss can be applied.
    ///
    /// # Errors
    /// This function will return an error if it could not create the default [`PagerState`] or fails
    /// to process the events
    pub(crate) fn generate_initial_state(
        rx: &Receiver<Command>,
        mut out: &mut Stdout,
    ) -> Result<Self, MinusError> {
        let mut ps = Self::new()?;
        rx.try_iter().try_for_each(|ev| -> Result<(), MinusError> {
            handle_event(
                ev,
                &mut out,
                &mut ps,
                &Arc::new(AtomicBool::new(false)),
                #[cfg(feature = "search")]
                &Arc::new((Mutex::new(true), Condvar::new())),
            )
        })?;
        Ok(ps)
    }

    pub(crate) fn format_lines(&mut self) {
        let format_result = text::make_format_lines(
            &self.screen.orig_text,
            self.line_numbers,
            self.cols,
            #[cfg(feature = "search")]
            &self.search_state.search_term,
        );
        #[cfg(feature = "search")]
        {
            self.search_state.search_idx = format_result.append_search_idx;
        }
        self.screen.formatted_lines = format_result.lines;
        self.lines_to_row_map = format_result.lines_to_row_map;

        self.unterminated = format_result.num_unterminated;
        self.format_prompt();
    }

    /// Reformat the inputted prompt to how it should be displayed
    pub(crate) fn format_prompt(&mut self) {
        const SEARCH_BG: &str = "\x1b[34m";
        const INPUT_BG: &str = "\x1b[33m";

        // Allocate the string. Add extra space in case for the
        // ANSI escape things if we do have characters typed and search showing
        let mut format_string = String::with_capacity(self.cols + (SEARCH_BG.len() * 2) + 4);

        // Get the string that will contain the search index/match indicator
        #[cfg(feature = "search")]
        let mut search_str = String::new();
        #[cfg(feature = "search")]
        if !self.search_state.search_idx.is_empty() {
            search_str.push(' ');
            search_str.push_str(&(self.search_state.search_mark + 1).to_string());
            search_str.push('/');
            search_str.push_str(&self.search_state.search_idx.len().to_string());
            search_str.push(' ');
        }

        // And get the string that will contain the prefix_num
        let mut prefix_str = String::new();
        if !self.prefix_num.is_empty() {
            prefix_str.push(' ');
            prefix_str.push_str(&self.prefix_num);
            prefix_str.push(' ');
        }

        // And lastly, the string that contains the prompt or msg
        let prompt_str = self.message.as_ref().unwrap_or(&self.prompt);

        #[cfg(feature = "search")]
        let search_len = search_str.len();
        #[cfg(not(feature = "search"))]
        let search_len = 0;

        // Calculate how much extra padding in the middle we need between
        // the prompt/message and the indicators on the right
        let prefix_len = prefix_str.len();
        let extra_space = self
            .cols
            .saturating_sub(search_len + prefix_len + prompt_str.len());
        let dsp_prompt: &str = if extra_space == 0 {
            &prompt_str[..self.cols - search_len - prefix_len]
        } else {
            prompt_str
        };

        // push the prompt/msg
        format_string.push_str(dsp_prompt);
        format_string.push_str(&" ".repeat(extra_space));

        // add the prefix_num if it exists
        if prefix_len > 0 {
            format_string.push_str(INPUT_BG);
            format_string.push_str(&prefix_str);
        }

        // and add the search indicator stuff if it exists
        #[cfg(feature = "search")]
        if search_len > 0 {
            format_string.push_str(SEARCH_BG);
            format_string.push_str(&search_str);
        }

        self.displayed_prompt = format_string;
    }

    /// Runs the exit callbacks
    pub(crate) fn exit(&mut self) {
        for func in &mut self.exit_callbacks {
            func();
        }
    }

    pub(crate) fn append_str(&mut self, text: &str) -> AppendStyle {
        let append = self.screen.orig_text.ends_with('\n') || self.screen.orig_text.is_empty();
        let attachment = if append {
            None
        } else {
            self.screen
                .orig_text
                .lines()
                .last()
                .map(ToString::to_string)
        };

        let prev_line_count = self.screen.orig_text.lines().count();
        let old_len_line_number = if prev_line_count == 0 {
            0
        } else {
            prev_line_count.ilog10() + 1
        };

        self.screen.orig_text.push_str(text);

        let new_line_count = self.screen.orig_text.lines().count();
        let new_len_line_number = if new_line_count == 0 {
            0
        } else {
            new_line_count.ilog10() + 1
        };

        let append_opts = text::FormatOpts {
            text,
            attachment,
            line_numbers: self.line_numbers,
            len_line_number: new_len_line_number.try_into().unwrap(),
            formatted_lines_count: self.screen.formatted_lines.len(),
            lines_count: prev_line_count,
            prev_unterminated: self.unterminated,
            cols: self.cols,
            #[cfg(feature = "search")]
            search_term: &self.search_state.search_term,
        };

        let append_props = text::format_text_block(append_opts);

        let (fmt_line, num_unterminated, lines_to_row_map) = (
            append_props.lines,
            append_props.num_unterminated,
            append_props.lines_to_row_map,
        );

        #[cfg(feature = "search")]
        {
            let mut append_search_idx = append_props.append_search_idx;
            self.search_state.search_idx.append(&mut append_search_idx);
        }
        self.lines_to_row_map.extend(lines_to_row_map);

        if new_len_line_number != old_len_line_number && old_len_line_number != 0 {
            self.format_lines();
            return AppendStyle::FullRedraw;
        }

        // Conditionally appends to [`self.formatted_lines`] or changes the last unterminated rows of
        // [`self.formatted_lines`]
        //
        // `num_unterminated` is the current number of lines returned by [`self.make_append_str`]
        // that should be truncated from [`self.formatted_lines`] to update the last line
        self.screen
            .formatted_lines
            .truncate(self.screen.formatted_lines.len() - self.unterminated);
        self.screen.formatted_lines.append(&mut fmt_line.clone());
        self.unterminated = num_unterminated;

        AppendStyle::PartialUpdate(fmt_line)
    }
}
