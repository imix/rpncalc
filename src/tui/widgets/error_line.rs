use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, msg: Option<&str>) {
    let paragraph = match msg {
        None => Paragraph::new(""),
        Some(text) => Paragraph::new(text.to_owned()).style(Style::default().fg(Color::Red)),
    };
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_error_line(msg: Option<&str>, width: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| render(f, f.area(), msg)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn row_content(buf: &ratatui::buffer::Buffer, row: u16) -> String {
        let width = buf.area().width;
        (0..width)
            .map(|x| buf.cell((x, row)).unwrap().symbol().to_string())
            .collect()
    }

    // AC 4: no error → completely blank row
    #[test]
    fn test_no_error_is_blank() {
        let buf = render_error_line(None, 40);
        let content = row_content(&buf, 0);
        assert!(
            content.chars().all(|c| c == ' '),
            "error line with no message should be all spaces: {:?}",
            content
        );
    }

    // AC 5: error message appears in the row
    #[test]
    fn test_error_message_appears() {
        let buf = render_error_line(Some("Stack underflow"), 40);
        let content = row_content(&buf, 0);
        assert!(
            content.contains("Stack underflow"),
            "error message should appear in row: {:?}",
            content
        );
    }

    // AC 5: error message is rendered in red
    #[test]
    fn test_error_message_is_red() {
        let buf = render_error_line(Some("x"), 40);
        let cell = buf.cell((0u16, 0u16)).unwrap();
        assert_eq!(cell.fg, Color::Red, "error message should be Red");
    }
}
