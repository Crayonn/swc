use swc_atoms::js_word;
use swc_common::{BytePos, Span};
use swc_css_ast::*;

use super::{input::ParserInput, PResult, Parser};
use crate::{
    error::{Error, ErrorKind},
    parser::{values_and_units::is_math_function, BlockContentsGrammar, Ctx},
    Parse,
};

impl<I> Parse<AtRule> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<AtRule> {
        // Consume the next input token. Create a new at-rule with its name set to the
        // value of the current input token, its prelude initially set to an empty list,
        // and its value initially set to nothing.
        let at_rule_span = self.input.cur_span();
        let at_keyword_name = match bump!(self) {
            Token::AtKeyword { value, raw } => (value, raw),
            _ => {
                unreachable!()
            }
        };
        let at_rule_name = if at_keyword_name.0.starts_with("--") {
            AtRuleName::DashedIdent(DashedIdent {
                span: Span::new(
                    at_rule_span.lo + BytePos(1),
                    at_rule_span.hi,
                    Default::default(),
                ),
                value: at_keyword_name.0,
                raw: Some(at_keyword_name.1),
            })
        } else {
            AtRuleName::Ident(Ident {
                span: Span::new(
                    at_rule_span.lo + BytePos(1),
                    at_rule_span.hi,
                    Default::default(),
                ),
                value: at_keyword_name.0,
                raw: Some(at_keyword_name.1),
            })
        };
        let mut at_rule = AtRule {
            span: span!(self, at_rule_span.lo),
            name: at_rule_name,
            prelude: None,
            block: None,
        };
        let lowercased_name = match &at_rule.name {
            AtRuleName::Ident(ident) => ident.value.to_ascii_lowercase(),
            AtRuleName::DashedIdent(dashed_ident) => dashed_ident.value.to_ascii_lowercase(),
        };
        let parse_prelude = |parser: &mut Parser<I>| -> PResult<Option<Box<AtRulePrelude>>> {
            match lowercased_name {
                js_word!("viewport")
                | js_word!("-ms-viewport")
                | js_word!("-o-viewport")
                | js_word!("font-face") => {
                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(None)
                }
                js_word!("charset") => {
                    parser.input.skip_ws();

                    let span = parser.input.cur_span();
                    let charset = match cur!(parser) {
                        tok!("string") => parser.parse()?,
                        _ => {
                            return Err(Error::new(span, ErrorKind::InvalidCharsetAtRule));
                        }
                    };

                    let prelude = AtRulePrelude::CharsetPrelude(charset);

                    parser.input.skip_ws();

                    if !is!(parser, ";") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("';' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("container") => {
                    parser.input.skip_ws();

                    let prelude = AtRulePrelude::ContainerPrelude(parser.parse()?);

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("counter-style") => {
                    parser.input.skip_ws();

                    let prelude = AtRulePrelude::CounterStylePrelude(parser.parse()?);

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("font-palette-values") => {
                    parser.input.skip_ws();

                    let prelude = AtRulePrelude::FontPaletteValuesPrelude(parser.parse()?);

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("font-feature-values") => {
                    parser.input.skip_ws();

                    let prelude = AtRulePrelude::FontFeatureValuesPrelude(parser.parse()?);

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("stylistic")
                | js_word!("historical-forms")
                | js_word!("styleset")
                | js_word!("character-variant")
                | js_word!("swash")
                | js_word!("ornaments")
                | js_word!("annotation")
                    if parser.ctx.in_font_feature_values_at_rule =>
                {
                    parser.input.skip_ws();

                    Ok(None)
                }
                js_word!("layer") => {
                    parser.input.skip_ws();

                    let prelude = if is!(parser, Ident) {
                        let mut name_list: Vec<LayerName> = vec![];

                        while is!(parser, Ident) {
                            name_list.push(parser.parse()?);

                            parser.input.skip_ws();

                            if is!(parser, ",") {
                                eat!(parser, ",");

                                parser.input.skip_ws();
                            }
                        }

                        if is!(parser, ";") {
                            let first = name_list[0].span;
                            let last = name_list[name_list.len() - 1].span;

                            Some(AtRulePrelude::LayerPrelude(LayerPrelude::NameList(
                                LayerNameList {
                                    name_list,
                                    span: Span::new(first.lo, last.hi, Default::default()),
                                },
                            )))
                        } else {
                            if name_list.len() > 1 {
                                let span = parser.input.cur_span();

                                return Err(Error::new(span, ErrorKind::Expected("';' token")));
                            }

                            Some(AtRulePrelude::LayerPrelude(LayerPrelude::Name(
                                name_list.remove(0),
                            )))
                        }
                    } else {
                        None
                    };

                    parser.input.skip_ws();

                    match prelude {
                        Some(AtRulePrelude::LayerPrelude(LayerPrelude::Name(_))) | None => {
                            if !is!(parser, "{") {
                                let span = parser.input.cur_span();

                                return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                            }
                        }
                        Some(AtRulePrelude::LayerPrelude(LayerPrelude::NameList(_))) => {
                            if !is!(parser, ";") {
                                let span = parser.input.cur_span();

                                return Err(Error::new(span, ErrorKind::Expected("';' token")));
                            }
                        }
                        _ => {
                            unreachable!();
                        }
                    }

                    Ok(prelude.map(Box::new))
                }
                js_word!("document") | js_word!("-moz-document") => {
                    parser.input.skip_ws();

                    let span = parser.input.cur_span();
                    let url_match_fn = parser.parse()?;
                    let mut matching_functions = vec![url_match_fn];

                    loop {
                        parser.input.skip_ws();

                        if !eat!(parser, ",") {
                            break;
                        }

                        parser.input.skip_ws();

                        matching_functions.push(parser.parse()?);
                    }

                    let prelude = AtRulePrelude::DocumentPrelude(DocumentPrelude {
                        span: span!(parser, span.lo),
                        matching_functions,
                    });

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("page") => {
                    parser.input.skip_ws();

                    let prelude = if !is!(parser, "{") {
                        Some(AtRulePrelude::PagePrelude(parser.parse()?))
                    } else {
                        None
                    };

                    parser.input.skip_ws();

                    Ok(prelude.map(Box::new))
                }
                js_word!("top-left-corner")
                | js_word!("top-left")
                | js_word!("top-center")
                | js_word!("top-right")
                | js_word!("top-right-corner")
                | js_word!("bottom-left-corner")
                | js_word!("bottom-left")
                | js_word!("bottom-center")
                | js_word!("bottom-right")
                | js_word!("bottom-right-corner")
                | js_word!("left-top")
                | js_word!("left-middle")
                | js_word!("left-bottom")
                | js_word!("right-top")
                | js_word!("right-middle")
                | js_word!("right-bottom")
                    if parser.ctx.in_page_at_rule =>
                {
                    parser.input.skip_ws();

                    Ok(None)
                }
                js_word!("property") => {
                    parser.input.skip_ws();

                    let prelude = AtRulePrelude::PropertyPrelude(parser.parse()?);

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("namespace") => {
                    parser.input.skip_ws();

                    let span = parser.input.cur_span();
                    let mut prefix = None;

                    if is!(parser, Ident) {
                        prefix = match cur!(parser) {
                            tok!("ident") => Some(parser.parse()?),
                            _ => {
                                unreachable!()
                            }
                        };

                        parser.input.skip_ws();
                    }

                    let uri = match cur!(parser) {
                        tok!("string") => NamespacePreludeUri::Str(parser.parse()?),
                        tok!("url") => NamespacePreludeUri::Url(parser.parse()?),
                        tok!("function") => NamespacePreludeUri::Url(parser.parse()?),
                        _ => {
                            let span = parser.input.cur_span();

                            return Err(Error::new(
                                span,
                                ErrorKind::Expected("string, url or function tokens"),
                            ));
                        }
                    };

                    let prelude = AtRulePrelude::NamespacePrelude(NamespacePrelude {
                        span: span!(parser, span.lo),
                        prefix,
                        uri: Box::new(uri),
                    });

                    if !is!(parser, ";") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("';' token")));
                    }

                    Ok(Some(Box::new(prelude)))
                }
                js_word!("color-profile") => {
                    parser.input.skip_ws();

                    let name = match cur!(parser) {
                        Token::Ident { value, .. } => {
                            if value.starts_with("--") {
                                ColorProfileName::DashedIdent(parser.parse()?)
                            } else {
                                ColorProfileName::Ident(parser.parse()?)
                            }
                        }
                        _ => {
                            let span = parser.input.cur_span();

                            return Err(Error::new(span, ErrorKind::Expected("ident")));
                        }
                    };

                    let prelude = Box::new(AtRulePrelude::ColorProfilePrelude(name));

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(prelude))
                }
                js_word!("nest") => {
                    parser.input.skip_ws();

                    let prelude = Box::new(AtRulePrelude::NestPrelude(parser.parse()?));

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(prelude))
                }
                js_word!("media") => {
                    parser.input.skip_ws();

                    let media = if !is!(parser, "{") {
                        let media_query_list = parser.parse()?;

                        Some(Box::new(AtRulePrelude::MediaPrelude(media_query_list)))
                    } else {
                        None
                    };

                    parser.input.skip_ws();

                    Ok(media)
                }
                js_word!("supports") => {
                    parser.input.skip_ws();

                    let prelude = Box::new(AtRulePrelude::SupportsPrelude(parser.parse()?));

                    parser.input.skip_ws();

                    Ok(Some(prelude))
                }
                js_word!("import") => {
                    parser.input.skip_ws();

                    let span = parser.input.cur_span();
                    let href = Box::new(match cur!(parser) {
                        tok!("string") => ImportPreludeHref::Str(parser.parse()?),
                        tok!("url") => ImportPreludeHref::Url(parser.parse()?),
                        tok!("function") => ImportPreludeHref::Url(parser.parse()?),
                        _ => {
                            return Err(Error::new(
                                span,
                                ErrorKind::Expected("string, url or function token"),
                            ))
                        }
                    });

                    parser.input.skip_ws();

                    let layer_name = match cur!(parser) {
                        Token::Ident { value, .. } if *value.to_ascii_lowercase() == *"layer" => {
                            let name = ImportPreludeLayerName::Ident(parser.parse()?);

                            parser.input.skip_ws();

                            Some(Box::new(name))
                        }
                        Token::Function { value, .. }
                            if *value.to_ascii_lowercase() == *"layer" =>
                        {
                            let ctx = Ctx {
                                in_import_at_rule: true,
                                block_contents_grammar: BlockContentsGrammar::DeclarationValue,
                                ..parser.ctx
                            };

                            let func = parser.with_ctx(ctx).parse_as::<Function>()?;
                            if func.value.len() != 1 {
                                parser.errors.push(Error::new(
                                    func.span,
                                    ErrorKind::Expected(
                                        "layer function inside @import expected to have exactly \
                                         one ident argument",
                                    ),
                                ));
                                None
                            } else if let ComponentValue::LayerName(LayerName {
                                name: name_raw,
                                ..
                            }) = &func.value[0]
                            {
                                parser.input.skip_ws();

                                if name_raw.is_empty() {
                                    parser.errors.push(Error::new(
                                        func.span,
                                        ErrorKind::Expected(
                                            "layer function inside @import expected to have \
                                             exactly one ident argument",
                                        ),
                                    ));
                                    None
                                } else {
                                    Some(Box::new(ImportPreludeLayerName::Function(func)))
                                }
                            } else {
                                parser.errors.push(Error::new(
                                    func.span,
                                    ErrorKind::Expected(
                                        "layer function inside @import expected to have exactly \
                                         one ident argument",
                                    ),
                                ));
                                None
                            }
                        }
                        _ => None,
                    };

                    let supports = match cur!(parser) {
                        Token::Function { value, .. }
                            if *value.to_ascii_lowercase() == *"supports" =>
                        {
                            bump!(parser);

                            parser.input.skip_ws();

                            let supports =
                                if is_case_insensitive_ident!(parser, "not") || is!(parser, "(") {
                                    ImportPreludeSupportsType::SupportsCondition(parser.parse()?)
                                } else {
                                    ImportPreludeSupportsType::Declaration(parser.parse()?)
                                };

                            expect!(parser, ")");

                            Some(Box::new(supports))
                        }
                        _ => None,
                    };

                    let media = if !is!(parser, ";") {
                        Some(parser.parse()?)
                    } else {
                        None
                    };

                    parser.input.skip_ws();

                    let prelude = Box::new(AtRulePrelude::ImportPrelude(ImportPrelude {
                        span: span!(parser, span.lo),
                        href,
                        layer_name,
                        supports,
                        media,
                    }));

                    if !is!(parser, ";") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("';' token")));
                    }

                    Ok(Some(prelude))
                }
                js_word!("keyframes")
                | js_word!("-webkit-keyframes")
                | js_word!("-moz-keyframes")
                | js_word!("-o-keyframes")
                | js_word!("-ms-keyframes") => {
                    parser.input.skip_ws();

                    let prelude = Box::new(AtRulePrelude::KeyframesPrelude(parser.parse()?));

                    parser.input.skip_ws();

                    if !is!(parser, "{") {
                        let span = parser.input.cur_span();

                        return Err(Error::new(span, ErrorKind::Expected("'{' token")));
                    }

                    Ok(Some(prelude))
                }
                js_word!("custom-media") => {
                    parser.input.skip_ws();

                    let prelude = Box::new(AtRulePrelude::CustomMediaPrelude(parser.parse()?));

                    parser.input.skip_ws();

                    Ok(Some(prelude))
                }
                _ => {
                    let span = parser.input.cur_span();

                    return Err(Error::new(span, ErrorKind::Ignore));
                }
            }
        };
        let parse_simple_block = |parser: &mut Parser<I>| -> PResult<SimpleBlock> {
            let ctx = match lowercased_name {
                js_word!("viewport")
                | js_word!("-o-viewport")
                | js_word!("-ms-viewport")
                | js_word!("font-face")
                | js_word!("font-palette-values")
                | js_word!("stylistic")
                | js_word!("historical-forms")
                | js_word!("styleset")
                | js_word!("character-variant")
                | js_word!("swash")
                | js_word!("ornaments")
                | js_word!("annotation")
                | js_word!("property")
                | js_word!("color-profile")
                | js_word!("counter-style")
                | js_word!("top-left-corner")
                | js_word!("top-left")
                | js_word!("top-center")
                | js_word!("top-right")
                | js_word!("top-right-corner")
                | js_word!("bottom-left-corner")
                | js_word!("bottom-left")
                | js_word!("bottom-center")
                | js_word!("bottom-right")
                | js_word!("bottom-right-corner")
                | js_word!("left-top")
                | js_word!("left-middle")
                | js_word!("left-bottom")
                | js_word!("right-top")
                | js_word!("right-middle")
                | js_word!("right-bottom") => Ctx {
                    block_contents_grammar: BlockContentsGrammar::DeclarationList,
                    ..parser.ctx
                },
                js_word!("font-feature-values") => Ctx {
                    in_font_feature_values_at_rule: true,
                    block_contents_grammar: BlockContentsGrammar::DeclarationList,
                    ..parser.ctx
                },
                js_word!("page") => Ctx {
                    in_page_at_rule: true,
                    block_contents_grammar: BlockContentsGrammar::DeclarationList,
                    ..parser.ctx
                },
                js_word!("layer") => Ctx {
                    block_contents_grammar: BlockContentsGrammar::Stylesheet,
                    ..parser.ctx
                },
                js_word!("media")
                | js_word!("supports")
                | js_word!("container")
                | js_word!("document")
                | js_word!("-moz-document") => match parser.ctx.block_contents_grammar {
                    BlockContentsGrammar::StyleBlock => Ctx {
                        in_container_at_rule: lowercased_name == js_word!("container"),
                        block_contents_grammar: BlockContentsGrammar::StyleBlock,
                        ..parser.ctx
                    },
                    _ => Ctx {
                        in_container_at_rule: lowercased_name == js_word!("container"),
                        block_contents_grammar: BlockContentsGrammar::Stylesheet,
                        ..parser.ctx
                    },
                },
                js_word!("nest") => Ctx {
                    block_contents_grammar: BlockContentsGrammar::StyleBlock,
                    ..parser.ctx
                },
                _ => Ctx {
                    block_contents_grammar: BlockContentsGrammar::NoGrammar,
                    ..parser.ctx
                },
            };
            let block = match lowercased_name {
                js_word!("keyframes")
                | js_word!("-moz-keyframes")
                | js_word!("-o-keyframes")
                | js_word!("-webkit-keyframes")
                | js_word!("-ms-keyframes")
                    if is!(parser, "{") =>
                {
                    let span_block = parser.input.cur_span();
                    let name = parser.input.bump().unwrap();
                    let mut block = SimpleBlock {
                        span: Default::default(),
                        name,
                        value: vec![],
                    };

                    parser.input.skip_ws();

                    loop {
                        if is!(parser, "}") {
                            break;
                        }

                        parser.input.skip_ws();

                        let keyframe_block: KeyframeBlock = parser.parse()?;

                        block
                            .value
                            .push(ComponentValue::KeyframeBlock(keyframe_block));

                        parser.input.skip_ws();
                    }

                    expect!(parser, "}");

                    block.span = span!(parser, span_block.lo);

                    block
                }
                _ => parser.with_ctx(ctx).parse_as::<SimpleBlock>()?,
            };

            Ok(block)
        };

        loop {
            // <EOF-token>
            // This is a parse error. Return the at-rule.
            if is!(self, EOF) {
                at_rule.span = span!(self, at_rule_span.lo);

                return Ok(at_rule);
            }

            match cur!(self) {
                // <semicolon-token>
                // Return the at-rule.
                tok!(";") => {
                    self.input.bump();

                    at_rule.span = span!(self, at_rule_span.lo);

                    return Ok(at_rule);
                }
                // <{-token>
                // Consume a simple block and assign it to the at-rule’s block. Return the at-rule.
                tok!("{") => {
                    let state = self.input.state();
                    let block = match parse_simple_block(self) {
                        Ok(simple_block) => simple_block,
                        Err(err) => {
                            if *err.kind() != ErrorKind::Ignore {
                                self.errors.push(err);
                            }

                            self.input.reset(&state);

                            let ctx = Ctx {
                                block_contents_grammar: BlockContentsGrammar::NoGrammar,
                                ..self.ctx
                            };

                            self.with_ctx(ctx).parse_as::<SimpleBlock>()?
                        }
                    };

                    at_rule.block = Some(block);
                    at_rule.span = span!(self, at_rule_span.lo);

                    return Ok(at_rule);
                }
                // anything else
                // Reconsume the current input token. Consume a component value. Append the returned
                // value to the at-rule’s prelude.
                _ => {
                    let state = self.input.state();

                    match parse_prelude(self) {
                        Ok(prelude) => {
                            if let Some(prelude) = prelude {
                                at_rule.prelude = Some(prelude);
                            }
                        }
                        Err(err) => {
                            if *err.kind() != ErrorKind::Ignore {
                                self.errors.push(err);
                            }

                            self.input.reset(&state);

                            let span = self.input.cur_span();

                            let mut list_of_component_value = match at_rule.prelude.as_deref_mut() {
                                Some(AtRulePrelude::ListOfComponentValues(
                                    ref mut list_of_component_value,
                                )) => list_of_component_value,
                                _ => {
                                    at_rule.prelude =
                                        Some(Box::new(AtRulePrelude::ListOfComponentValues(
                                            ListOfComponentValues {
                                                span: span!(self, span.lo),
                                                children: vec![],
                                            },
                                        )));

                                    match at_rule.prelude.as_deref_mut() {
                                        Some(AtRulePrelude::ListOfComponentValues(
                                            ref mut list_of_component_value,
                                        )) => list_of_component_value,
                                        _ => {
                                            unreachable!();
                                        }
                                    }
                                }
                            };

                            let ctx = Ctx {
                                block_contents_grammar: BlockContentsGrammar::NoGrammar,
                                ..self.ctx
                            };
                            let component_value =
                                self.with_ctx(ctx).parse_as::<ComponentValue>()?;

                            list_of_component_value.children.push(component_value);
                            list_of_component_value.span = Span::new(
                                list_of_component_value.span.lo,
                                span.hi,
                                Default::default(),
                            );
                        }
                    }
                }
            }
        }
    }
}

impl<I> Parse<KeyframesName> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<KeyframesName> {
        match cur!(self) {
            tok!(":") if self.config.css_modules => {
                let span = self.input.cur_span();

                bump!(self);

                match cur!(self) {
                    Token::Function { value, .. }
                        if (&*value.to_ascii_lowercase() == "local"
                            || &*value.to_ascii_lowercase() == "global") =>
                    {
                        let span = self.input.cur_span();
                        let pseudo = match bump!(self) {
                            Token::Function { value, raw, .. } => Ident {
                                span: span!(self, span.lo),
                                value,
                                raw: Some(raw),
                            },
                            _ => {
                                unreachable!();
                            }
                        };

                        self.input.skip_ws();

                        let name = self.parse()?;

                        self.input.skip_ws();

                        expect!(self, ")");

                        Ok(KeyframesName::PseudoFunction(Box::new(
                            KeyframesPseudoFunction {
                                span: span!(self, span.lo),
                                pseudo,
                                name,
                            },
                        )))
                    }
                    Token::Ident { value, .. }
                        if (&*value.to_ascii_lowercase() == "local"
                            || &*value.to_ascii_lowercase() == "global") =>
                    {
                        let pseudo = self.parse()?;

                        self.input.skip_ws();

                        let name = self.parse()?;

                        Ok(KeyframesName::PseudoPrefix(Box::new(
                            KeyframesPseudoPrefix {
                                span: span!(self, span.lo),
                                pseudo,
                                name,
                            },
                        )))
                    }
                    _ => {
                        let span = self.input.cur_span();

                        Err(Error::new(
                            span,
                            ErrorKind::Expected("ident or function (local or scope) token"),
                        ))
                    }
                }
            }
            tok!("ident") => {
                let custom_ident: CustomIdent = self.parse()?;

                if &*custom_ident.value.to_ascii_lowercase() == "none" {
                    return Err(Error::new(
                        custom_ident.span,
                        ErrorKind::InvalidCustomIdent(custom_ident.value),
                    ));
                }

                Ok(KeyframesName::CustomIdent(Box::new(custom_ident)))
            }
            tok!("string") => Ok(KeyframesName::Str(Box::new(self.parse()?))),
            _ => {
                let span = self.input.cur_span();

                Err(Error::new(span, ErrorKind::Expected("ident or string")))
            }
        }
    }
}

impl<I> Parse<FontFeatureValuesPrelude> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<FontFeatureValuesPrelude> {
        let span = self.input.cur_span();

        let mut font_family = vec![self.parse()?];

        loop {
            self.input.skip_ws();

            if !eat!(self, ",") {
                break;
            }

            self.input.skip_ws();

            font_family.push(self.parse()?);
        }

        Ok(FontFeatureValuesPrelude {
            span: span!(self, span.lo),
            font_family,
        })
    }
}

impl<I> Parse<KeyframeBlock> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<KeyframeBlock> {
        let span = self.input.cur_span();

        let child = self.parse()?;
        let mut prelude = vec![child];

        loop {
            self.input.skip_ws();

            if !eat!(self, ",") {
                break;
            }

            self.input.skip_ws();

            let child = self.parse()?;

            prelude.push(child);
        }

        let ctx = Ctx {
            block_contents_grammar: BlockContentsGrammar::DeclarationList,
            ..self.ctx
        };
        let block = self.with_ctx(ctx).parse_as::<SimpleBlock>()?;

        Ok(KeyframeBlock {
            span: span!(self, span.lo),
            prelude,
            block,
        })
    }
}

impl<I> Parse<KeyframeSelector> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<KeyframeSelector> {
        match cur!(self) {
            tok!("ident") => {
                let ident: Ident = self.parse()?;
                let normalized_ident_value = ident.value.to_ascii_lowercase();

                if &*normalized_ident_value != "from" && &*normalized_ident_value != "to" {
                    return Err(Error::new(
                        ident.span,
                        ErrorKind::Expected("'from' or 'to' idents"),
                    ));
                }

                Ok(KeyframeSelector::Ident(ident))
            }
            tok!("percentage") => Ok(KeyframeSelector::Percentage(self.parse()?)),
            _ => {
                let span = self.input.cur_span();

                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident or percentage token"),
                ));
            }
        }
    }
}

impl<I> Parse<SupportsCondition> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SupportsCondition> {
        self.input.skip_ws();

        let start_pos = self.input.cur_span().lo;
        let mut last_pos;
        let mut conditions = vec![];

        if is_case_insensitive_ident!(self, "not") {
            let not = self.parse()?;

            last_pos = self.input.last_pos();

            conditions.push(SupportsConditionType::Not(not));
        } else {
            let supports_in_parens = self.parse()?;

            last_pos = self.input.last_pos();

            conditions.push(SupportsConditionType::SupportsInParens(supports_in_parens));

            self.input.skip_ws();

            if is_case_insensitive_ident!(self, "and") {
                while is_case_insensitive_ident!(self, "and") {
                    let and = self.parse()?;

                    last_pos = self.input.last_pos();

                    conditions.push(SupportsConditionType::And(and));

                    self.input.skip_ws();
                }
            } else if is_case_insensitive_ident!(self, "or") {
                while is_case_insensitive_ident!(self, "or") {
                    let or = self.parse()?;

                    last_pos = self.input.last_pos();

                    conditions.push(SupportsConditionType::Or(or));

                    self.input.skip_ws();
                }
            }
        };

        Ok(SupportsCondition {
            span: Span::new(start_pos, last_pos, Default::default()),
            conditions,
        })
    }
}

impl<I> Parse<SupportsNot> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SupportsNot> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("not") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'not' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let supports_in_parens = self.parse()?;

        Ok(SupportsNot {
            span: span!(self, span.lo),
            keyword,
            condition: supports_in_parens,
        })
    }
}

impl<I> Parse<SupportsAnd> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SupportsAnd> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("and") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'and' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let supports_in_parens = self.parse()?;

        Ok(SupportsAnd {
            span: span!(self, span.lo),
            keyword,
            condition: supports_in_parens,
        })
    }
}

impl<I> Parse<SupportsOr> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SupportsOr> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("or") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'or' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let supports_in_parens = self.parse()?;

        Ok(SupportsOr {
            span: span!(self, span.lo),
            keyword,
            condition: supports_in_parens,
        })
    }
}

impl<I> Parse<SupportsInParens> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SupportsInParens> {
        let state = self.input.state();

        match self.parse() {
            Ok(feature) => Ok(SupportsInParens::Feature(feature)),
            Err(_) => {
                self.input.reset(&state);

                let mut parse_condition = || {
                    expect!(self, "(");

                    let condition = self.parse()?;

                    expect!(self, ")");

                    Ok(SupportsInParens::SupportsCondition(condition))
                };

                match parse_condition() {
                    Ok(condition) => Ok(condition),
                    Err(_) => {
                        self.input.reset(&state);

                        match self.parse() {
                            Ok(general_enclosed) => {
                                Ok(SupportsInParens::GeneralEnclosed(general_enclosed))
                            }
                            Err(err) => Err(err),
                        }
                    }
                }
            }
        }
    }
}

impl<I> Parse<SupportsFeature> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SupportsFeature> {
        match cur!(self) {
            tok!("(") => {
                bump!(self);

                self.input.skip_ws();

                let declaration = self.parse()?;

                self.input.skip_ws();

                expect!(self, ")");

                Ok(SupportsFeature::Declaration(declaration))
            }
            Token::Function { value, .. } if &*value.to_ascii_lowercase() == "selector" => {
                // TODO improve me
                let ctx = Ctx {
                    block_contents_grammar: BlockContentsGrammar::DeclarationValue,
                    in_supports_at_rule: true,
                    ..self.ctx
                };
                let function = self.with_ctx(ctx).parse_as::<Function>()?;

                Ok(SupportsFeature::Function(function))
            }
            _ => {
                let span = self.input.cur_span();

                Err(Error::new(
                    span,
                    ErrorKind::Expected("'(' or 'function' token"),
                ))
            }
        }
    }
}

impl<I> Parse<GeneralEnclosed> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<GeneralEnclosed> {
        match cur!(self) {
            tok!("function") => {
                let ctx = Ctx {
                    block_contents_grammar: BlockContentsGrammar::NoGrammar,
                    ..self.ctx
                };

                let function = self.with_ctx(ctx).parse_as::<Function>()?;

                Ok(GeneralEnclosed::Function(function))
            }
            tok!("(") => {
                let ctx = Ctx {
                    block_contents_grammar: BlockContentsGrammar::NoGrammar,
                    ..self.ctx
                };
                let block = self.with_ctx(ctx).parse_as::<SimpleBlock>()?;

                if let Some(first) = block.value.get(0) {
                    match first {
                        ComponentValue::PreservedToken(token_and_span) => {
                            match token_and_span.token {
                                Token::Ident { .. } => {}
                                _ => {
                                    return Err(Error::new(
                                        block.span,
                                        ErrorKind::Expected(
                                            "ident token at first position in <general-enclosed>",
                                        ),
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(Error::new(
                                block.span,
                                ErrorKind::Expected(
                                    "ident token at first position in <general-enclosed>",
                                ),
                            ));
                        }
                    }
                }

                Ok(GeneralEnclosed::SimpleBlock(block))
            }
            _ => {
                let span = self.input.cur_span();

                Err(Error::new(
                    span,
                    ErrorKind::Expected("function or '(' token"),
                ))
            }
        }
    }
}

impl<I> Parse<DocumentPreludeMatchingFunction> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<DocumentPreludeMatchingFunction> {
        match cur!(self) {
            tok!("url") => Ok(DocumentPreludeMatchingFunction::Url(self.parse()?)),
            Token::Function {
                value: function_name,
                ..
            } => {
                if &*function_name.to_ascii_lowercase() == "url"
                    || &*function_name.to_ascii_lowercase() == "src"
                {
                    Ok(DocumentPreludeMatchingFunction::Url(self.parse()?))
                } else {
                    // TODO improve me
                    let ctx = Ctx {
                        block_contents_grammar: BlockContentsGrammar::DeclarationValue,
                        ..self.ctx
                    };

                    let function = self.with_ctx(ctx).parse_as::<Function>()?;

                    Ok(DocumentPreludeMatchingFunction::Function(function))
                }
            }
            _ => {
                let span = self.input.cur_span();

                Err(Error::new(span, ErrorKind::Expected("url or function")))
            }
        }
    }
}

impl<I> Parse<MediaQueryList> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaQueryList> {
        self.input.skip_ws();

        let query = self.parse()?;
        let mut queries = vec![query];

        // TODO error recovery
        // To parse a <media-query-list> production, parse a comma-separated list of
        // component values, then parse each entry in the returned list as a
        // <media-query>. Its value is the list of <media-query>s so produced.
        loop {
            self.input.skip_ws();

            if !eat!(self, ",") {
                break;
            }

            self.input.skip_ws();

            let query = self.parse()?;

            queries.push(query);
        }

        let start_pos = match queries.first() {
            Some(MediaQuery { span, .. }) => span.lo,
            _ => {
                unreachable!();
            }
        };
        let last_pos = match queries.last() {
            Some(MediaQuery { span, .. }) => span.hi,
            _ => {
                unreachable!();
            }
        };

        Ok(MediaQueryList {
            span: Span::new(start_pos, last_pos, Default::default()),
            queries,
        })
    }
}

impl<I> Parse<MediaQuery> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaQuery> {
        let start_pos = self.input.cur_span().lo;
        let state = self.input.state();

        let is_not = is_one_of_case_insensitive_ident!(self, "not");
        let modifier = if is_one_of_case_insensitive_ident!(self, "not", "only") {
            let modifier = Some(self.parse()?);

            self.input.skip_ws();

            modifier
        } else {
            None
        };

        if is!(self, "ident") {
            let media_type = Some(self.parse()?);

            self.input.skip_ws();

            let mut keyword = None;
            let mut condition_without_or = None;

            if is_one_of_case_insensitive_ident!(self, "and") {
                keyword = Some(self.parse()?);

                self.input.skip_ws();

                condition_without_or = Some(Box::new(MediaConditionType::WithoutOr(self.parse()?)));
            }

            let end_pos = if let Some(MediaConditionType::WithoutOr(condition_without_or)) =
                condition_without_or.as_deref()
            {
                condition_without_or.span.hi
            } else if let Some(MediaType::Ident(ident)) = &media_type {
                ident.span.hi
            } else {
                unreachable!();
            };

            return Ok(MediaQuery {
                span: Span::new(start_pos, end_pos, Default::default()),
                modifier,
                media_type,
                keyword,
                condition: condition_without_or,
            });
        }

        if is_not {
            self.input.reset(&state);
        }

        let condition: MediaCondition = self.parse()?;

        Ok(MediaQuery {
            span: Span::new(start_pos, condition.span.hi, Default::default()),
            modifier: None,
            media_type: None,
            keyword: None,
            condition: Some(Box::new(MediaConditionType::All(condition))),
        })
    }
}

impl<I> Parse<MediaType> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaType> {
        match cur!(self) {
            _ if !is_one_of_case_insensitive_ident!(self, "not", "and", "or", "only", "layer") => {
                Ok(MediaType::Ident(self.parse()?))
            }
            _ => {
                let span = self.input.cur_span();

                Err(Error::new(
                    span,
                    ErrorKind::Expected(
                        "ident (exclude the keywords 'only', 'not', 'and', 'or' and 'layer')",
                    ),
                ))
            }
        }
    }
}

impl<I> Parse<MediaCondition> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaCondition> {
        self.input.skip_ws();

        let start_pos = self.input.cur_span().lo;
        let mut last_pos;
        let mut conditions = vec![];

        if is_case_insensitive_ident!(self, "not") {
            let not = self.parse()?;

            last_pos = self.input.last_pos();

            conditions.push(MediaConditionAllType::Not(not));
        } else {
            let media_in_parens = self.parse()?;

            last_pos = self.input.last_pos();

            conditions.push(MediaConditionAllType::MediaInParens(media_in_parens));

            self.input.skip_ws();

            if is_case_insensitive_ident!(self, "and") {
                while is_case_insensitive_ident!(self, "and") {
                    let and = self.parse()?;

                    last_pos = self.input.last_pos();

                    conditions.push(MediaConditionAllType::And(and));

                    self.input.skip_ws();
                }
            } else if is_case_insensitive_ident!(self, "or") {
                while is_case_insensitive_ident!(self, "or") {
                    let or = self.parse()?;

                    last_pos = self.input.last_pos();

                    conditions.push(MediaConditionAllType::Or(or));

                    self.input.skip_ws();
                }
            }
        };

        Ok(MediaCondition {
            span: Span::new(start_pos, last_pos, Default::default()),
            conditions,
        })
    }
}

impl<I> Parse<MediaConditionWithoutOr> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaConditionWithoutOr> {
        self.input.skip_ws();

        let start_pos = self.input.cur_span().lo;
        let mut last_pos;
        let mut conditions = vec![];

        if is_case_insensitive_ident!(self, "not") {
            let not = self.parse()?;

            last_pos = self.input.last_pos();

            conditions.push(MediaConditionWithoutOrType::Not(not));
        } else {
            let media_in_parens = self.parse()?;

            last_pos = self.input.last_pos();

            conditions.push(MediaConditionWithoutOrType::MediaInParens(media_in_parens));

            self.input.skip_ws();

            if is_case_insensitive_ident!(self, "and") {
                while is_case_insensitive_ident!(self, "and") {
                    let and = self.parse()?;

                    last_pos = self.input.last_pos();

                    conditions.push(MediaConditionWithoutOrType::And(and));

                    self.input.skip_ws();
                }
            }
        };

        Ok(MediaConditionWithoutOr {
            span: Span::new(start_pos, last_pos, Default::default()),
            conditions,
        })
    }
}

impl<I> Parse<MediaNot> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaNot> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("not") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'not' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let media_in_parens = self.parse()?;

        Ok(MediaNot {
            span: span!(self, span.lo),
            keyword,
            condition: media_in_parens,
        })
    }
}

impl<I> Parse<MediaAnd> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaAnd> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("and") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'and' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let media_in_parens = self.parse()?;

        Ok(MediaAnd {
            span: span!(self, span.lo),
            keyword,
            condition: media_in_parens,
        })
    }
}

impl<I> Parse<MediaOr> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaOr> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("or") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'or' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let media_in_parens = self.parse()?;

        Ok(MediaOr {
            span: span!(self, span.lo),
            keyword,
            condition: media_in_parens,
        })
    }
}

impl<I> Parse<MediaInParens> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaInParens> {
        let state = self.input.state();

        match self.parse() {
            Ok(media_feature) => Ok(MediaInParens::Feature(media_feature)),
            Err(_) => {
                self.input.reset(&state);

                let mut parse_media_condition = || {
                    expect!(self, "(");

                    let media_condition = self.parse()?;

                    expect!(self, ")");

                    Ok(MediaInParens::MediaCondition(media_condition))
                };

                match parse_media_condition() {
                    Ok(media_in_parens) => Ok(media_in_parens),
                    Err(_) => {
                        self.input.reset(&state);

                        let general_enclosed = self.parse()?;

                        Ok(MediaInParens::GeneralEnclosed(general_enclosed))
                    }
                }
            }
        }
    }
}

impl<I> Parse<MediaFeature> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaFeature> {
        let span = self.input.cur_span();

        expect!(self, "(");

        self.input.skip_ws();

        let left = self.parse()?;

        self.input.skip_ws();

        match cur!(self) {
            tok!(")") => {
                bump!(self);

                let name = match left {
                    MediaFeatureValue::Ident(ident) => MediaFeatureName::Ident(ident),
                    _ => {
                        return Err(Error::new(span, ErrorKind::Expected("identifier value")));
                    }
                };

                Ok(MediaFeature::Boolean(MediaFeatureBoolean {
                    span: span!(self, span.lo),
                    name,
                }))
            }
            tok!(":") => {
                bump!(self);

                self.input.skip_ws();

                let name = match left {
                    MediaFeatureValue::Ident(ident) => MediaFeatureName::Ident(ident),
                    _ => {
                        return Err(Error::new(span, ErrorKind::Expected("identifier value")));
                    }
                };
                let value = self.parse()?;

                self.input.skip_ws();

                expect!(self, ")");

                Ok(MediaFeature::Plain(MediaFeaturePlain {
                    span: span!(self, span.lo),
                    name,
                    value,
                }))
            }
            tok!("<") | tok!(">") | tok!("=") => {
                let left_comparison = match bump!(self) {
                    tok!("<") => {
                        if eat!(self, "=") {
                            MediaFeatureRangeComparison::Le
                        } else {
                            MediaFeatureRangeComparison::Lt
                        }
                    }
                    tok!(">") => {
                        if eat!(self, "=") {
                            MediaFeatureRangeComparison::Ge
                        } else {
                            MediaFeatureRangeComparison::Gt
                        }
                    }
                    tok!("=") => MediaFeatureRangeComparison::Eq,
                    _ => {
                        unreachable!();
                    }
                };

                self.input.skip_ws();

                let center = self.parse()?;

                self.input.skip_ws();

                if eat!(self, ")") {
                    return Ok(MediaFeature::Range(MediaFeatureRange {
                        span: span!(self, span.lo),
                        left: Box::new(left),
                        comparison: left_comparison,
                        right: Box::new(center),
                    }));
                }

                let right_comparison = match bump!(self) {
                    tok!("<") => {
                        if eat!(self, "=") {
                            MediaFeatureRangeComparison::Le
                        } else {
                            MediaFeatureRangeComparison::Lt
                        }
                    }
                    tok!(">") => {
                        if eat!(self, "=") {
                            MediaFeatureRangeComparison::Ge
                        } else {
                            MediaFeatureRangeComparison::Gt
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            span,
                            ErrorKind::Expected("'>' or '<' operators"),
                        ));
                    }
                };

                self.input.skip_ws();

                let right = self.parse()?;

                self.input.skip_ws();

                expect!(self, ")");

                let name = match center {
                    MediaFeatureValue::Ident(ident) => MediaFeatureName::Ident(ident),
                    _ => {
                        return Err(Error::new(span, ErrorKind::Expected("identifier value")));
                    }
                };

                let is_valid_operator = match left_comparison {
                    MediaFeatureRangeComparison::Lt | MediaFeatureRangeComparison::Le
                        if right_comparison == MediaFeatureRangeComparison::Lt
                            || right_comparison == MediaFeatureRangeComparison::Le =>
                    {
                        true
                    }
                    MediaFeatureRangeComparison::Gt | MediaFeatureRangeComparison::Ge
                        if right_comparison == MediaFeatureRangeComparison::Gt
                            || right_comparison == MediaFeatureRangeComparison::Ge =>
                    {
                        true
                    }
                    _ => false,
                };

                if !is_valid_operator {
                    return Err(Error::new(
                        span,
                        ErrorKind::Expected(
                            "left comparison operator should be equal right comparison operator",
                        ),
                    ));
                }

                Ok(MediaFeature::RangeInterval(MediaFeatureRangeInterval {
                    span: span!(self, span.lo),
                    left: Box::new(left),
                    left_comparison,
                    name,
                    right_comparison,
                    right,
                }))
            }
            _ => Err(Error::new(span, ErrorKind::Expected("identifier value"))),
        }
    }
}

impl<I> Parse<MediaFeatureValue> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<MediaFeatureValue> {
        let span = self.input.cur_span();

        match cur!(self) {
            tok!("number") => {
                let left = self.parse()?;

                self.input.skip_ws();

                if eat!(self, "/") {
                    self.input.skip_ws();

                    let right = Some(self.parse()?);

                    return Ok(MediaFeatureValue::Ratio(Ratio {
                        span: span!(self, span.lo),
                        left,
                        right,
                    }));
                }

                Ok(MediaFeatureValue::Number(left))
            }
            tok!("ident") => Ok(MediaFeatureValue::Ident(self.parse()?)),
            tok!("dimension") => Ok(MediaFeatureValue::Dimension(self.parse()?)),
            Token::Function { value, .. } if is_math_function(value) => {
                let ctx = Ctx {
                    block_contents_grammar: BlockContentsGrammar::DeclarationValue,
                    ..self.ctx
                };

                Ok(MediaFeatureValue::Function(
                    self.with_ctx(ctx).parse_as::<Function>()?,
                ))
            }
            _ => Err(Error::new(
                span,
                ErrorKind::Expected("number, ident, dimension or function token"),
            )),
        }
    }
}

impl<I> Parse<PageSelectorList> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<PageSelectorList> {
        let selector = self.parse()?;
        let mut selectors = vec![selector];

        loop {
            self.input.skip_ws();

            if !eat!(self, ",") {
                break;
            }

            self.input.skip_ws();

            let selector = self.parse()?;

            selectors.push(selector);
        }

        let start_pos = match selectors.first() {
            Some(PageSelector { span, .. }) => span.lo,
            _ => {
                unreachable!();
            }
        };
        let last_pos = match selectors.last() {
            Some(PageSelector { span, .. }) => span.hi,
            _ => {
                unreachable!();
            }
        };

        Ok(PageSelectorList {
            span: Span::new(start_pos, last_pos, Default::default()),
            selectors,
        })
    }
}

impl<I> Parse<PageSelector> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<PageSelector> {
        self.input.skip_ws();

        let span = self.input.cur_span();

        let page_type = if is!(self, Ident) {
            Some(self.parse()?)
        } else {
            None
        };

        let pseudos = if is!(self, ":") {
            let mut pseudos = vec![];

            loop {
                if !is!(self, ":") {
                    break;
                }

                let pseudo = self.parse()?;

                pseudos.push(pseudo);
            }

            Some(pseudos)
        } else {
            None
        };

        Ok(PageSelector {
            span: span!(self, span.lo),
            page_type,
            pseudos,
        })
    }
}

impl<I> Parse<PageSelectorType> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<PageSelectorType> {
        let span = self.input.cur_span();
        let value = self.parse()?;

        Ok(PageSelectorType {
            span: span!(self, span.lo),
            value,
        })
    }
}

impl<I> Parse<PageSelectorPseudo> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<PageSelectorPseudo> {
        let span = self.input.cur_span();

        expect!(self, ":");

        let value = match cur!(self) {
            Token::Ident { value, .. }
                if matches!(
                    &*value.to_ascii_lowercase(),
                    "left" | "right" | "first" | "blank"
                ) =>
            {
                self.parse()?
            }
            _ => {
                let span = self.input.cur_span();

                return Err(Error::new(
                    span,
                    ErrorKind::Expected("'left', 'right', 'first' or 'blank' ident"),
                ));
            }
        };

        Ok(PageSelectorPseudo {
            span: span!(self, span.lo),
            value,
        })
    }
}

impl<I> Parse<LayerName> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<LayerName> {
        let start = self.input.cur_span().lo;
        let mut name = vec![];

        let entered = is!(self, Ident);

        while is!(self, Ident) {
            let span = self.input.cur_span();
            let token = bump!(self);
            let ident = match token {
                Token::Ident { value, raw } => Ident {
                    span,
                    value,
                    raw: Some(raw),
                },
                _ => {
                    unreachable!();
                }
            };

            name.push(ident);

            if is!(self, ".") {
                eat!(self, ".");
            }
        }

        if !entered {
            // if first argument is not ident, without bump! will cause infinite loop
            bump!(self);
        }

        Ok(LayerName {
            name,
            span: span!(self, start),
        })
    }
}

impl<I> Parse<ContainerCondition> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<ContainerCondition> {
        let start_pos = self.input.cur_span().lo;

        let mut name: Option<ContainerName> = None;

        if is!(self, "ident") && !is_case_insensitive_ident!(self, "not") {
            name = Some(self.parse()?);

            self.input.skip_ws();
        }

        let query: ContainerQuery = self.parse()?;

        Ok(ContainerCondition {
            span: Span::new(start_pos, query.span.hi, Default::default()),
            name,
            query,
        })
    }
}

impl<I> Parse<ContainerName> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<ContainerName> {
        match cur!(self) {
            tok!("ident") => {
                let custom_ident: CustomIdent = self.parse()?;

                Ok(ContainerName::CustomIdent(custom_ident))
            }
            _ => {
                let span = self.input.cur_span();

                Err(Error::new(span, ErrorKind::Expected("ident")))
            }
        }
    }
}

impl<I> Parse<ContainerQuery> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<ContainerQuery> {
        let start_pos = self.input.cur_span().lo;
        let mut last_pos;

        let mut queries = vec![];

        if is_case_insensitive_ident!(self, "not") {
            let not = self.parse()?;

            queries.push(ContainerQueryType::Not(not));

            last_pos = self.input.last_pos();
        } else {
            self.input.skip_ws();

            let query_in_parens = self.parse()?;

            queries.push(ContainerQueryType::QueryInParens(query_in_parens));

            last_pos = self.input.last_pos();

            self.input.skip_ws();

            if is_case_insensitive_ident!(self, "and") {
                while is_case_insensitive_ident!(self, "and") {
                    let and = self.parse()?;

                    last_pos = self.input.last_pos();

                    queries.push(ContainerQueryType::And(and));

                    self.input.skip_ws();
                }
            } else if is_case_insensitive_ident!(self, "or") {
                while is_case_insensitive_ident!(self, "or") {
                    let or = self.parse()?;

                    last_pos = self.input.last_pos();

                    queries.push(ContainerQueryType::Or(or));

                    self.input.skip_ws();
                }
            };
        }

        Ok(ContainerQuery {
            span: Span::new(start_pos, last_pos, Default::default()),
            queries,
        })
    }
}

impl<I> Parse<ContainerQueryNot> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<ContainerQueryNot> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("not") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'not' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let query_in_parens = self.parse()?;

        Ok(ContainerQueryNot {
            span: span!(self, span.lo),
            keyword,
            query: query_in_parens,
        })
    }
}

impl<I> Parse<ContainerQueryAnd> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<ContainerQueryAnd> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("and") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'and' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let query_in_parens = self.parse()?;

        Ok(ContainerQueryAnd {
            span: span!(self, span.lo),
            keyword,
            query: query_in_parens,
        })
    }
}

impl<I> Parse<ContainerQueryOr> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<ContainerQueryOr> {
        let span = self.input.cur_span();
        let keyword = match cur!(self) {
            Token::Ident { value, .. } if value.as_ref().eq_ignore_ascii_case("or") => {
                Some(self.parse()?)
            }
            _ => {
                return Err(Error::new(
                    span,
                    ErrorKind::Expected("ident (with 'or' value) token"),
                ));
            }
        };

        self.input.skip_ws();

        let query_in_parens = self.parse()?;

        Ok(ContainerQueryOr {
            span: span!(self, span.lo),
            keyword,
            query: query_in_parens,
        })
    }
}

impl<I> Parse<QueryInParens> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<QueryInParens> {
        let state = self.input.state();

        match self.parse() {
            Ok(size_feature) => Ok(QueryInParens::SizeFeature(size_feature)),
            Err(_) => {
                self.input.reset(&state);

                let mut parse_container_query = || {
                    expect!(self, "(");

                    let container_query = self.parse()?;

                    expect!(self, ")");

                    Ok(QueryInParens::ContainerQuery(Box::new(container_query)))
                };

                match parse_container_query() {
                    Ok(query_in_parens) => Ok(query_in_parens),
                    Err(_) => {
                        self.input.reset(&state);

                        let general_enclosed = self.parse()?;

                        Ok(QueryInParens::GeneralEnclosed(general_enclosed))
                    }
                }
            }
        }
    }
}

impl<I> Parse<SizeFeature> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SizeFeature> {
        let span = self.input.cur_span();

        expect!(self, "(");

        self.input.skip_ws();

        let left = self.parse()?;

        self.input.skip_ws();

        match cur!(self) {
            tok!(")") => {
                bump!(self);

                let name = match left {
                    SizeFeatureValue::Ident(ident) => SizeFeatureName::Ident(ident),
                    _ => {
                        return Err(Error::new(span, ErrorKind::Expected("identifier value")));
                    }
                };

                Ok(SizeFeature::Boolean(SizeFeatureBoolean {
                    span: span!(self, span.lo),
                    name,
                }))
            }
            tok!(":") => {
                bump!(self);

                self.input.skip_ws();

                let name = match left {
                    SizeFeatureValue::Ident(ident) => SizeFeatureName::Ident(ident),
                    _ => {
                        return Err(Error::new(span, ErrorKind::Expected("identifier value")));
                    }
                };
                let value = self.parse()?;

                self.input.skip_ws();

                expect!(self, ")");

                Ok(SizeFeature::Plain(SizeFeaturePlain {
                    span: span!(self, span.lo),
                    name,
                    value,
                }))
            }
            tok!("<") | tok!(">") | tok!("=") => {
                let left_comparison = match bump!(self) {
                    tok!("<") => {
                        if eat!(self, "=") {
                            SizeFeatureRangeComparison::Le
                        } else {
                            SizeFeatureRangeComparison::Lt
                        }
                    }
                    tok!(">") => {
                        if eat!(self, "=") {
                            SizeFeatureRangeComparison::Ge
                        } else {
                            SizeFeatureRangeComparison::Gt
                        }
                    }
                    tok!("=") => SizeFeatureRangeComparison::Eq,
                    _ => {
                        unreachable!();
                    }
                };

                self.input.skip_ws();

                let center = self.parse()?;

                self.input.skip_ws();

                if eat!(self, ")") {
                    return Ok(SizeFeature::Range(SizeFeatureRange {
                        span: span!(self, span.lo),
                        left: Box::new(left),
                        comparison: left_comparison,
                        right: Box::new(center),
                    }));
                }

                let right_comparison = match bump!(self) {
                    tok!("<") => {
                        if eat!(self, "=") {
                            SizeFeatureRangeComparison::Le
                        } else {
                            SizeFeatureRangeComparison::Lt
                        }
                    }
                    tok!(">") => {
                        if eat!(self, "=") {
                            SizeFeatureRangeComparison::Ge
                        } else {
                            SizeFeatureRangeComparison::Gt
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            span,
                            ErrorKind::Expected("'>' or '<' operators"),
                        ));
                    }
                };

                self.input.skip_ws();

                let right = self.parse()?;

                self.input.skip_ws();

                expect!(self, ")");

                let name = match center {
                    SizeFeatureValue::Ident(ident) => SizeFeatureName::Ident(ident),
                    _ => {
                        return Err(Error::new(span, ErrorKind::Expected("identifier value")));
                    }
                };

                let is_valid_operator = match left_comparison {
                    SizeFeatureRangeComparison::Lt | SizeFeatureRangeComparison::Le
                        if right_comparison == SizeFeatureRangeComparison::Lt
                            || right_comparison == SizeFeatureRangeComparison::Le =>
                    {
                        true
                    }
                    SizeFeatureRangeComparison::Gt | SizeFeatureRangeComparison::Ge
                        if right_comparison == SizeFeatureRangeComparison::Gt
                            || right_comparison == SizeFeatureRangeComparison::Ge =>
                    {
                        true
                    }
                    _ => false,
                };

                if !is_valid_operator {
                    return Err(Error::new(
                        span,
                        ErrorKind::Expected(
                            "left comparison operator should be equal right comparison operator",
                        ),
                    ));
                }

                Ok(SizeFeature::RangeInterval(SizeFeatureRangeInterval {
                    span: span!(self, span.lo),
                    left: Box::new(left),
                    left_comparison,
                    name,
                    right_comparison,
                    right,
                }))
            }
            _ => Err(Error::new(span, ErrorKind::Expected("identifier value"))),
        }
    }
}

impl<I> Parse<SizeFeatureValue> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<SizeFeatureValue> {
        let span = self.input.cur_span();

        match cur!(self) {
            tok!("number") => {
                let left = self.parse()?;

                self.input.skip_ws();

                if eat!(self, "/") {
                    self.input.skip_ws();

                    let right = Some(self.parse()?);

                    return Ok(SizeFeatureValue::Ratio(Ratio {
                        span: span!(self, span.lo),
                        left,
                        right,
                    }));
                }

                Ok(SizeFeatureValue::Number(left))
            }
            tok!("ident") => Ok(SizeFeatureValue::Ident(self.parse()?)),
            tok!("dimension") => Ok(SizeFeatureValue::Dimension(self.parse()?)),
            Token::Function { value, .. } if is_math_function(value) => {
                let ctx = Ctx {
                    block_contents_grammar: BlockContentsGrammar::DeclarationValue,
                    ..self.ctx
                };

                Ok(SizeFeatureValue::Function(
                    self.with_ctx(ctx).parse_as::<Function>()?,
                ))
            }
            _ => Err(Error::new(
                span,
                ErrorKind::Expected("number, ident, dimension or function token"),
            )),
        }
    }
}

impl<I> Parse<ExtensionName> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<ExtensionName> {
        let span = self.input.cur_span();

        if !is!(self, Ident) {
            return Err(Error::new(span, ErrorKind::Expected("indent token")));
        }

        match bump!(self) {
            Token::Ident { value, raw } => {
                if !value.starts_with("--") {
                    return Err(Error::new(
                        span,
                        ErrorKind::Expected("Extension name should start with '--'"),
                    ));
                }

                Ok(ExtensionName {
                    span,
                    value,
                    raw: Some(raw),
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}

impl<I> Parse<CustomMediaQuery> for Parser<I>
where
    I: ParserInput,
{
    fn parse(&mut self) -> PResult<CustomMediaQuery> {
        let span = self.input.cur_span();
        let name = self.parse()?;

        self.input.skip_ws();

        let media = match cur!(self) {
            _ if is_case_insensitive_ident!(self, "true")
                || is_case_insensitive_ident!(self, "false") =>
            {
                CustomMediaQueryMediaType::Ident(self.parse()?)
            }
            _ => CustomMediaQueryMediaType::MediaQueryList(self.parse()?),
        };

        Ok(CustomMediaQuery {
            span: span!(self, span.lo),
            name,
            media,
        })
    }
}