use ariadne::Color;
use ariadne::Config;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use ariadne::Source;
use chumsky::prelude::*;

fn report_error(
    fname: &str,
    msg: String,
    label: (String, SimpleSpan),
    extra_labels: impl IntoIterator<Item = (String, SimpleSpan)>,
    src: &str,
) {
    Report::build(ReportKind::Error, (fname, label.1.into_range()))
        .with_config(Config::new().with_index_type(ariadne::IndexType::Byte))
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
        .print((fname, Source::from(src)))
        .unwrap()
}

pub fn report_errors(fname: &str, errs: &[Rich<impl std::fmt::Display>], src: &str) -> ! {
    for err in errs {
        report_error(
            fname,
            err.reason().to_string(),
            (
                err.found()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "end of input".to_string()),
                *err.span(),
            ),
            err.contexts()
                .map(|(l, s)| (format!("while parsing this {l}"), *s)),
            src,
        );
    }
    std::process::exit(1)
}
