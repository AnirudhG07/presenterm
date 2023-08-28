use crate::{
    elements::{Element, Text, TextChunk, TextFormat},
    slide::Slide,
};
use comrak::{
    nodes::{AstNode, NodeHeading, NodeValue},
    parse_document, Arena, ComrakOptions,
};
use std::mem;

type ParseResult<T> = Result<T, ParseError>;

pub struct SlideParser<'a> {
    arena: &'a Arena<AstNode<'a>>,
    options: ComrakOptions,
}

impl<'a> SlideParser<'a> {
    pub fn new(arena: &'a Arena<AstNode<'a>>, options: ComrakOptions) -> Self {
        Self { arena, options }
    }

    pub fn parse(&self, document: &str) -> ParseResult<Vec<Slide>> {
        let root = parse_document(self.arena, document, &self.options);
        let mut slides = Vec::new();
        let mut slide_elements = Vec::new();
        for node in root.children() {
            let value = &node.data.borrow().value;
            match value {
                NodeValue::ThematicBreak => {
                    let slide = Slide::new(mem::take(&mut slide_elements));
                    slides.push(slide);
                    continue;
                }
                _ => {
                    slide_elements.push(Self::parse_element(node)?);
                }
            };
        }
        if !slide_elements.is_empty() {
            slides.push(Slide::new(slide_elements));
        }
        Ok(slides)
    }

    fn parse_element(node: &'a AstNode<'a>) -> ParseResult<Element> {
        let value = &node.data.borrow().value;
        let element = match value {
            NodeValue::Heading(heading) => Self::parse_heading(heading, node)?,
            NodeValue::Paragraph => Self::parse_paragraph(node)?,
            other => return Err(ParseError::UnsupportedElement(other.identifier())),
        };
        Ok(element)
    }

    fn parse_heading(heading: &NodeHeading, node: &'a AstNode<'a>) -> ParseResult<Element> {
        let text = Self::parse_text(node)?;
        let element = Element::Heading { text, level: heading.level };
        Ok(element)
    }

    fn parse_paragraph(node: &'a AstNode<'a>) -> ParseResult<Element> {
        let text = Self::parse_text(node)?;
        let element = Element::Paragraph { text };
        Ok(element)
    }

    fn parse_text(root: &'a AstNode<'a>) -> ParseResult<Text> {
        let chunks = Self::parse_text_chunks(root, TextFormat::default())?;
        Ok(Text { chunks })
    }

    fn parse_text_chunks(root: &'a AstNode<'a>, format: TextFormat) -> ParseResult<Vec<TextChunk>> {
        let mut chunks = Vec::new();
        for node in root.children() {
            let value = &node.data.borrow().value;
            match value {
                NodeValue::Text(text) => {
                    chunks.push(TextChunk::formatted(text.clone(), format.clone()));
                }
                NodeValue::Strong => chunks.extend(Self::parse_text_chunks(node, format.clone().add_bold())?),
                NodeValue::Emph => chunks.extend(Self::parse_text_chunks(node, format.clone().add_italics())?),
                other => {
                    return Err(ParseError::UnsupportedStructure { container: "text", element: other.identifier() })
                }
            };
        }
        Ok(chunks)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("unsupported element: {0}")]
    UnsupportedElement(&'static str),

    #[error("unsupported structure in {container}: {element}")]
    UnsupportedStructure { container: &'static str, element: &'static str },
}

trait Identifier {
    fn identifier(&self) -> &'static str;
}

impl Identifier for NodeValue {
    fn identifier(&self) -> &'static str {
        match self {
            NodeValue::Document => "document",
            NodeValue::FrontMatter(_) => "front matter",
            NodeValue::BlockQuote => "block quote",
            NodeValue::List(_) => "list",
            NodeValue::Item(_) => "item",
            NodeValue::DescriptionList => "description list",
            NodeValue::DescriptionItem(_) => "description item",
            NodeValue::DescriptionTerm => "description term",
            NodeValue::DescriptionDetails => "description details",
            NodeValue::CodeBlock(_) => "code block",
            NodeValue::HtmlBlock(_) => "html block",
            NodeValue::Paragraph => "paragraph",
            NodeValue::Heading(_) => "heading",
            NodeValue::ThematicBreak => "thematic break",
            NodeValue::FootnoteDefinition(_) => "footnote definition",
            NodeValue::Table(_) => "table",
            NodeValue::TableRow(_) => "table row",
            NodeValue::TableCell => "table cell",
            NodeValue::Text(_) => "text",
            NodeValue::TaskItem(_) => "task item",
            NodeValue::SoftBreak => "soft break",
            NodeValue::LineBreak => "line break",
            NodeValue::Code(_) => "code",
            NodeValue::HtmlInline(_) => "inline html",
            NodeValue::Emph => "emph",
            NodeValue::Strong => "strong",
            NodeValue::Strikethrough => "strikethrough",
            NodeValue::Superscript => "superscript",
            NodeValue::Link(_) => "link",
            NodeValue::Image(_) => "image",
            NodeValue::FootnoteReference(_) => "footnote reference",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse_single(input: &str) -> Element {
        let arena = Arena::new();
        let root = parse_document(&arena, input, &ComrakOptions::default());
        assert_eq!(root.children().count(), 1, "expected a single child");

        let result = SlideParser::parse_element(root.first_child().unwrap()).expect("parsing failed");
        result
    }

    fn parse_slides(input: &str) -> Vec<Slide> {
        let arena = Arena::new();
        let parser = SlideParser::new(&arena, ComrakOptions::default());
        parser.parse(input).expect("parsing failed")
    }

    #[test]
    fn paragraph() {
        let parsed = parse_single("some **bold text**, _italics_, *italics*, **nested _italics_**");
        let Element::Paragraph { text } = parsed else { panic!("not a paragraph: {parsed:?}"); };
        let expected_chunks = [
            TextChunk::unformatted("some "),
            TextChunk::formatted("bold text", TextFormat::default().add_bold()),
            TextChunk::unformatted(", "),
            TextChunk::formatted("italics", TextFormat::default().add_italics()),
            TextChunk::unformatted(", "),
            TextChunk::formatted("italics", TextFormat::default().add_italics()),
            TextChunk::unformatted(", "),
            TextChunk::formatted("nested ", TextFormat::default().add_bold()),
            TextChunk::formatted("italics", TextFormat::default().add_italics().add_bold()),
        ];
        assert_eq!(text.chunks, expected_chunks);
    }

    #[test]
    fn heading() {
        let parsed = parse_single("# Title **with bold**");
        let Element::Heading { text, level } = parsed else { panic!("not a heading: {parsed:?}"); };
        let expected_chunks =
            [TextChunk::unformatted("Title "), TextChunk::formatted("with bold", TextFormat::default().add_bold())];

        assert_eq!(level, 1);
        assert_eq!(text.chunks, expected_chunks);
    }

    #[test]
    fn slide_splitting() {
        let slides = parse_slides(
            "First

---
Second

***
Third
",
        );
        assert_eq!(slides.len(), 3);

        assert_eq!(slides[0].elements.len(), 1);
        assert_eq!(slides[1].elements.len(), 1);
        assert_eq!(slides[2].elements.len(), 1);

        let expected = ["First", "Second", "Third"];
        for (slide, expected) in slides.into_iter().zip(expected) {
            let Element::Paragraph{ text } = &slide.elements[0] else { panic!("no text") };
            let chunks = [TextChunk::unformatted(expected)];
            assert_eq!(text.chunks, chunks);
        }
    }
}