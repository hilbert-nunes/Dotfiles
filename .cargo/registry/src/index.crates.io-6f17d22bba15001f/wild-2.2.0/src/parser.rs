use std::fmt;

/// An experimental, low-level access to each individual character of raw arguments.
#[must_use]
pub struct CommandLineWParser<'argsline> {
    line: std::slice::Iter<'argsline, u16>,
}

impl<'argsline> CommandLineWParser<'argsline> {
    #[inline]
    #[must_use]
    pub fn new(command_line_args_ucs2: &'argsline [u16]) -> Self {
        Self {
            line: command_line_args_ucs2.iter(),
        }
    }
}

impl<'a> fmt::Debug for CommandLineWParser<'a> {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        String::from_utf16_lossy(self.line.as_slice()).fmt(f)
    }
}

#[derive(Debug)]
enum State {
    BetweenArgs,
    InArg(bool),
    OnQuote,
    Backslashes(usize, bool),
}

/// A single code unit, which may be UCS-2 or half-broken UTF-16. Not a character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharCode {
    /// This code unit was found inside quotes (it's just text)
    Quoted(u16),
    /// This code unit was found outside quotes (you could interpret it as a glob)
    Unquoted(u16),
}

/// Given UCS2/potentially-broken-UTF-16 string parses one argument, following
/// the absolutely bizarre quoting rules of `CommandLineToArgvW`, and returns
/// a bool indicating whether there's anything more left.
///
/// Calling this repeatedly until it returns false will parse all arguments.
///
/// The callback is expected to accumulate code units itself.
///
/// This parses u16 code units, rather than code points.
/// This allows supporting unpaired surrogates and ensures they won't "eat" any control characters.
impl<'argsline> CommandLineWParser<'argsline> {
    pub fn accumulate_next<CharacterAccumulator>(&mut self, mut push: CharacterAccumulator) -> bool
        where CharacterAccumulator: FnMut(CharCode)
    {
        use self::State::*;
        let mut state = BetweenArgs;
        for &cu in &mut self.line {
            state = match state {
                BetweenArgs => match cu {
                    c if c == u16::from(b' ') => BetweenArgs,
                    c if c == u16::from(b'"') => InArg(true),
                    c if c == u16::from(b'\\') => Backslashes(1, false),
                    c => {
                        push(CharCode::Unquoted(c));
                        InArg(false)
                    },
                },
                InArg(quoted) => match cu {
                    c if c == u16::from(b'\\') => Backslashes(1, quoted),
                    c if quoted && c == u16::from(b'"') => OnQuote,
                    c if !quoted && c == u16::from(b'"') => InArg(true),
                    c if !quoted && c == u16::from(b' ') => {
                        return true;
                    },
                    c => {
                        push(if quoted { CharCode::Quoted(c) } else { CharCode::Unquoted(c) });
                        InArg(quoted)
                    },
                },
                OnQuote => match cu {
                    c if c == u16::from(b'"') => {
                        // In quoted arg "" means literal quote and the end of the quoted string (but not arg)
                        push(CharCode::Quoted(u16::from(b'"')));
                        InArg(false)
                    },
                    c if c == u16::from(b' ') => {
                        return true;
                    },
                    c => {
                        push(CharCode::Unquoted(c));
                        InArg(false)
                    },
                },
                Backslashes(count, quoted) => match cu {
                    c if c == u16::from(b'\\') => Backslashes(count + 1, quoted),
                    c if c == u16::from(b'"') => {
                        // backslashes followed by a quotation mark are treated as pairs of protected backslashes
                        let b = if quoted { CharCode::Quoted(u16::from(b'\\')) } else { CharCode::Unquoted(u16::from(b'\\')) };
                        for _ in 0..count/2 {
                            push(b);
                        }

                        if count & 1 != 0 {
                            // An odd number of backslashes is treated as followed by a protected quotation mark.
                            let c = u16::from(b'"');
                            push(if quoted { CharCode::Quoted(c) } else { CharCode::Unquoted(c) });
                            InArg(quoted)
                        } else if quoted {
                            // An even number of backslashes is treated as followed by a word terminator.
                            return true;
                        } else {
                            InArg(quoted)
                        }
                    },
                    c => {
                        // A string of backslashes not followed by a quotation mark has no special meaning.
                        let b = if quoted { CharCode::Quoted(u16::from(b'\\')) } else { CharCode::Unquoted(u16::from(b'\\')) };
                        for _ in 0..count {
                            push(b);
                        }
                        push(if quoted { CharCode::Quoted(c) } else { CharCode::Unquoted(c) });
                        InArg(quoted)
                    },
                },
            }
        }
        match state {
            BetweenArgs => false,
            OnQuote | InArg(..) => true,
            Backslashes(count, quoted) => {
                // A string of backslashes not followed by a quotation mark has no special meaning.
                let b = if quoted { CharCode::Quoted(u16::from(b'\\')) } else { CharCode::Unquoted(u16::from(b'\\')) };
                for _ in 0..count {
                    push(b);
                }
                true
            },
        }
    }
}
