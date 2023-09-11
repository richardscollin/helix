use std::borrow::Cow;
use log::warn;

use crate::{Editor, Document, View};

fn expand_one_variable(view: &View, doc: &Document, variable: &str) -> anyhow::Result<String> {
    match variable {
        "basename" => Ok(doc
            .path()
            .and_then(|it| it.file_name().and_then(|it| it.to_str()))
            .unwrap_or(crate::document::SCRATCH_BUFFER_NAME)
            .to_owned()),
        "filename" => Ok(doc
            .path()
            .and_then(|it| it.to_str())
            .unwrap_or(crate::document::SCRATCH_BUFFER_NAME)
            .to_owned()),
        "dirname" => Ok(doc
            .path()
            .and_then(|p| p.parent())
            .and_then(std::path::Path::to_str)
            .unwrap_or(std::env::current_dir()?.to_str().unwrap())
            .to_owned()),
        "linenumber" => Ok((doc
            .selection(view.id)
            .primary()
            .cursor_line(doc.text().slice(..)) + 1)
            .to_string()),
        "selection" => Ok(doc
            .selection(view.id)
            .primary()
            .fragment(doc.text().slice(..))
            .to_string()),
        _ => anyhow::bail!("Unknown variable"),
    }
}


pub fn expand_variables<'a>(editor: &Editor, input: &'a str) -> anyhow::Result<Cow<'a, str>> {
    if input.find('%').is_none() {
        // no expansion in this case

        // this special case handling could be removed
        return Ok(std::borrow::Cow::Borrowed(input));
    }

    let (view, doc) = current_ref!(editor);
    let _shell = &editor.config().shell;

    const RESERVE_CAPACITY: usize = 32; // extra capacity for expected expansion memory
    let mut output: String = String::with_capacity(input.len() + RESERVE_CAPACITY);

    const SH_VAR_PREFIX: &str = "%sh{";
    const VAR_PREFIX: &str = "%{";

    let mut remaining = input;
    while !remaining.is_empty() {
        if remaining.starts_with(SH_VAR_PREFIX) {
            remaining = &remaining[SH_VAR_PREFIX.len()..];
            // TODO

        } else if remaining.starts_with(VAR_PREFIX) {
            remaining = &remaining[VAR_PREFIX.len()..];
            if let Some(closing_brace_location) = remaining.find('}') {
                let expanded = expand_one_variable(view, doc, &remaining[..closing_brace_location]);
                output.push_str(expanded?.trim());
                remaining = &remaining[(closing_brace_location+1)..];
            } else {
                // missing closing brace
                warn!("missing closing brace");
                output.push_str(remaining);
                break;
            }
        } else if remaining.starts_with('%') {
            // if we didn't match above
            // consume a single char and append to output
            output.push('%');
            remaining = &remaining[1..];
        } else {
            // scan until next %
            let loc = remaining.find('%').unwrap_or(remaining.len());
            output.push_str(&remaining[..loc]);
            remaining = &remaining[loc..];
        }
    }

    Ok(std::borrow::Cow::Owned(output))
}

/*
pub fn expand_variables2<'a>(editor: &Editor, input: &'a str) -> anyhow::Result<Cow<'a, str>> {
    let (view, doc) = current_ref!(editor);
    let shell = &editor.config().shell;

    let mut output: Option<String> = None;

    let mut chars = input.char_indices();
    let mut last_push_end: usize = 0;

    while let Some((index, char)) = chars.next() {
        if char == '%' {
            if let Some((_, char)) = chars.next() {
                if char == '{' {
                    for (end, char) in chars.by_ref() {
                        if char == '}' {
                            if output.is_none() {
                                output = Some(String::with_capacity(input.len()))
                            }

                            if let Some(o) = output.as_mut() {
                                o.push_str(&input[last_push_end..index]);
                                last_push_end = end + 1;

                                //vlet value = expand(&doc, &input[index + 2..end]);

                                o.push_str(value.trim());

                                break;
                            }
                        }
                    }
                } else if char == 's' {
                    if let (Some((_, 'h')), Some((_, '{'))) = (chars.next(), chars.next()) {
                        let mut right_bracket_remaining = 1;
                        for (end, char) in chars.by_ref() {
                            if char == '}' {
                                right_bracket_remaining -= 1;

                                if right_bracket_remaining == 0 {
                                    if output.is_none() {
                                        output = Some(String::with_capacity(input.len()))
                                    }

                                    if let Some(o) = output.as_mut() {
                                        let body =
                                            expand_variables(editor, &input[index + 4..end])?;

                                        let output = tokio::task::block_in_place(move || {
                                            helix_lsp::block_on(async move {
                                                let mut command =
                                                    tokio::process::Command::new(&shell[0]);
                                                command.args(&shell[1..]).arg(&body[..]);

                                                let output =
                                                    command.output().await.map_err(|_| {
                                                        anyhow::anyhow!(
                                                            "Shell command failed: {body}"
                                                        )
                                                    })?;

                                                if output.status.success() {
                                                    String::from_utf8(output.stdout).map_err(|_| {
                                                        anyhow::anyhow!(
                                                            "Process did not output valid UTF-8"
                                                        )
                                                    })
                                                } else if output.stderr.is_empty() {
                                                    Err(anyhow::anyhow!(
                                                        "Shell command failed: {body}"
                                                    ))
                                                } else {
                                                    let stderr =
                                                        String::from_utf8_lossy(&output.stderr);

                                                    Err(anyhow::anyhow!("{stderr}"))
                                                }
                                            })
                                        });
                                        o.push_str(&input[last_push_end..index]);
                                        last_push_end = end + 1;

                                        o.push_str(output?.trim());

                                        break;
                                    }
                                }
                            } else if char == '{' {
                                right_bracket_remaining += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(o) = output.as_mut() {
        o.push_str(&input[last_push_end..]);
    }

    match output {
        Some(o) => Ok(std::borrow::Cow::Owned(o)),
        None => Ok(std::borrow::Cow::Borrowed(input)),
    }
}
*/
