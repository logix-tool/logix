use std::cmp::Ordering;

use logix::{based_path::BasedPath, error::Error};
use owo_colors::OwoColorize;
use prettydiff::basic::DiffOp;

use crate::{main_utils::theme::DiffTheme, Context};

struct Ln<'a> {
    a: Vec<&'a str>,
    last_a: usize,
    b: Vec<&'a str>,
    last_b: usize,
    ctx: &'a Context,
    theme: &'a DiffTheme,
}

impl<'a> Ln<'a> {
    fn print_both(&mut self) {
        self.last_a = self.a.len();
        self.last_b = self.b.len();
        write!(
            self.ctx,
            "{} |{} | ",
            format!("{:>4}", self.a.len()).color(self.theme.line_a),
            format!("{:>4}", self.b.len()).color(self.theme.line_b)
        );
    }

    fn print_b(&mut self) {
        self.last_b = self.b.len();
        write!(
            self.ctx,
            "     |{} | ",
            format!("{:>4}", self.b.len()).color(self.theme.line_b)
        );
    }

    fn print_a(&mut self) {
        self.last_a = self.a.len();
        write!(
            self.ctx,
            "{} |     | ",
            format!("{:>4}", self.a.len()).color(self.theme.line_a)
        );
    }

    fn print_context(&mut self) {
        let a = self.a[self.last_a..].iter().next_back().copied();
        let b = self.b[self.last_b..].iter().next_back().copied();
        if a == b {
            if let Some(a) = a {
                write!(self.ctx, "  |");
                self.print_both();
                writeln!(self.ctx, "{a}");
            }
        }
    }

    fn print_lines(&mut self, lines: &[&'a str], added: bool) {
        self.print_context();
        for v in lines {
            if added {
                self.b.push(v);
                write!(self.ctx, "{} |", "A".color(self.theme.added));
                self.print_b();
                write!(self.ctx, "{}", v.color(self.theme.added));
            } else {
                self.a.push(v);
                write!(self.ctx, "{} |", "D".color(self.theme.removed));
                self.print_a();
                write!(self.ctx, "{}", v.color(self.theme.removed));
            }
            writeln!(self.ctx);
        }
    }
}

pub fn diff_text_files(ctx: &Context, local: &BasedPath, logix: &BasedPath) -> Result<(), Error> {
    let theme = &ctx.theme;

    let local_file = std::fs::read_to_string(local)
        .map_err(|e| Error::ReadForDiff(local.clone(), e.to_string()))?;
    let logix_file = std::fs::read_to_string(logix)
        .map_err(|e| Error::ReadForDiff(logix.clone(), e.to_string()))?;

    let file_diff = prettydiff::text::diff_lines(&logix_file, &local_file);

    let mut ln = Ln {
        a: Vec::new(),
        last_a: 0,
        b: Vec::new(),
        last_b: 0,
        ctx,
        theme: &ctx.theme.diff,
    };

    for op in file_diff.diff() {
        match op {
            DiffOp::Insert(v) => {
                ln.print_lines(v, true);
                writeln!(ctx);
            }
            DiffOp::Remove(v) => {
                ln.print_lines(v, false);
                writeln!(ctx);
            }
            DiffOp::Replace(a, b) => {
                ln.print_context();

                for (a, b) in a.iter().copied().zip(b.iter().copied()) {
                    let diff = prettydiff::text::diff_words(a, b);
                    let diff = diff.diff();

                    if diff.iter().all(|op| {
                        matches!(
                            op,
                            DiffOp::Insert(..) | DiffOp::Remove(..) | DiffOp::Equal(..)
                        )
                    }) {
                        write!(ctx, "{} |", "M".color(theme.diff.modified));

                        ln.a.push(a);
                        ln.b.push(b);

                        ln.print_both();

                        for op in diff {
                            match op {
                                DiffOp::Insert(v) => {
                                    for v in v {
                                        write!(ctx, "{}", v.color(theme.diff.added));
                                    }
                                }
                                DiffOp::Replace(..) => unreachable!(),
                                DiffOp::Remove(v) => {
                                    for v in v {
                                        write!(ctx, "{}", v.color(theme.diff.removed));
                                    }
                                }
                                DiffOp::Equal(v) => {
                                    for v in v {
                                        write!(ctx, "{}", v);
                                    }
                                }
                            }
                        }
                        writeln!(ctx);
                    } else {
                        ln.print_lines(&[a], false);
                        ln.print_lines(&[b], true);
                    }
                }

                match a.len().cmp(&b.len()) {
                    Ordering::Equal => {}
                    Ordering::Less => {
                        ln.print_lines(&b[a.len()..], true);
                    }
                    Ordering::Greater => {
                        ln.print_lines(&a[b.len()..], false);
                    }
                }

                writeln!(ctx);
            }
            DiffOp::Equal(v) => {
                for v in v {
                    ln.a.push(v);
                    ln.b.push(v);
                }
            }
        }
    }
    Ok(())
}
