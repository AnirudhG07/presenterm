use crate::{
    elements::{Element, Text},
    slide::Slide,
};
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal::{self, ClearType},
    QueueableCommand,
};
use std::io::{self, Write};

pub struct Drawer {
    handle: io::Stdout,
}

impl Drawer {
    pub fn new() -> io::Result<Self> {
        let mut handle = io::stdout();
        handle.queue(cursor::Hide)?;
        Ok(Self { handle })
    }

    pub fn draw(&mut self, slides: &[Slide]) -> io::Result<()> {
        self.handle.queue(terminal::Clear(ClearType::All))?;
        self.handle.queue(cursor::MoveTo(0, 0))?;

        self.draw_slide(&slides[0])
    }

    fn draw_slide(&mut self, slide: &Slide) -> io::Result<()> {
        for element in &slide.elements {
            self.draw_element(element)?;
        }
        self.handle.flush()?;
        Ok(())
    }

    fn draw_element(&mut self, element: &Element) -> io::Result<()> {
        self.handle.queue(cursor::MoveToColumn(0))?;
        match element {
            // TODO handle level
            Element::Heading { text, .. } => {
                self.handle.queue(style::SetAttribute(style::Attribute::Bold))?;
                self.draw_text(text)?;
                self.handle.queue(cursor::MoveDown(2))?;
                self.handle.queue(style::SetAttribute(style::Attribute::Reset))?;
            }
            Element::Paragraph { text } => {
                self.draw_text(text)?;
                self.handle.queue(cursor::MoveDown(2))?;
            }
        };
        Ok(())
    }

    fn draw_text(&mut self, text: &Text) -> io::Result<()> {
        for chunk in &text.chunks {
            let mut styled = chunk.text.clone().stylize();
            if chunk.format.has_bold() {
                styled = styled.bold();
            }
            if chunk.format.has_italics() {
                styled = styled.italic();
            }
            self.handle.queue(style::PrintStyledContent(styled))?;
        }
        Ok(())
    }
}