use ariadne::Color;
use ariadne::Config;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use chumsky::error::RichReason;
use chumsky::prelude::*;
use std::fmt::Display;
use std::io::IsTerminal;

fn error_message(err: &Rich<impl Display>) -> String {
    match err.reason() {
        RichReason::Custom(msg) => msg.clone(),
        RichReason::ExpectedFound { expected, found } => {
            let found = found
                .as_deref()
                .map(|token| format!("'{token}'"))
                .unwrap_or_else(|| "end of input".to_string());
            let mut expected: Vec<_> = expected.iter().map(ToString::to_string).collect();
            expected.sort();
            expected.dedup();
            let expected = match expected.as_slice() {
                [] => return format!("Unexpected {found}"),
                [one] => one.clone(),
                [one, two] => format!("{one} or {two}"),
                many => {
                    let (last, rest) = many.split_last().unwrap();
                    format!("{}, or {last}", rest.join(", "))
                }
            };
            format!("Expected {expected}, found {found}")
        }
    }
}

fn report_error(
    fname: &str,
    msg: String,
    label: (String, SimpleSpan),
    extra_labels: impl IntoIterator<Item = (String, SimpleSpan)>,
    src: &str,
) {
    Report::build(ReportKind::Error, (fname, label.1.into_range()))
        .with_config(
            Config::new()
                .with_index_type(ariadne::IndexType::Byte)
                .with_color(std::io::stderr().is_terminal()),
        )
        .with_message(&msg)
        .with_label(
            Label::new((fname, label.1.into_range()))
                .with_message(label.0)
                .with_color(Color::Red),
        )
        .with_labels(extra_labels.into_iter().map(|label2| {
            Label::new((fname, label2.1.into_range()))
                .with_message(label2.0)
                .with_color(Color::Yellow)
        }))
        .finish()
        .eprint((fname, Source::from(src)))
        .unwrap()
}

pub fn report_errors(fname: &str, errs: &[Rich<impl std::fmt::Display>], src: &str) -> ! {
    for err in errs {
        report_error(
            fname,
            error_message(err),
            (
                match err.reason() {
                    RichReason::Custom(_) => "here".to_string(),
                    RichReason::ExpectedFound {
                        found: Some(token), ..
                    } => {
                        format!("unexpected '{}'", **token)
                    }
                    RichReason::ExpectedFound { found: None, .. } => "input ends here".to_string(),
                },
                *err.span(),
            ),
            err.contexts()
                .map(|(l, s)| (format!("while parsing this {l}"), *s)),
            src,
        );
    }
    std::process::exit(1)
}
