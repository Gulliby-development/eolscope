pub fn show(value: &str) -> String {
    format!("\"{}\"", plain(value))
}

pub fn plain(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if (' '..='~').contains(&character) {
                character
            } else {
                '?'
            }
        })
        .collect()
}

pub fn plural(n: usize, word: &str) -> String {
    let suffix = if n == 1 { "" } else { "s" };
    format!("{n} {word}{suffix}")
}

pub fn table(header: &[&str], rows: &[Vec<String>]) -> Vec<String> {
    let column_count = rows
        .iter()
        .fold(header.len(), |count, row| count.max(row.len()));
    let mut widths = vec![0; column_count];

    for (column, cell) in header.iter().enumerate() {
        if let Some(width) = widths.get_mut(column) {
            *width = cell.len();
        }
    }

    for row in rows {
        for (column, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(column) {
                *width = (*width).max(cell.len());
            }
        }
    }

    let mut lines = Vec::new();
    lines.push(render_row(header.iter().copied(), &widths));
    for row in rows {
        lines.push(render_row(row.iter().map(String::as_str), &widths));
    }
    lines
}

fn render_row<'a>(cells: impl Iterator<Item = &'a str>, widths: &[usize]) -> String {
    let mut cells = cells;
    let mut line = String::new();

    for (column, width) in widths.iter().enumerate() {
        let cell = cells.next().unwrap_or_default();
        line.push_str(cell);

        if column + 1 < widths.len() {
            for _ in cell.len()..*width {
                line.push(' ');
            }
            line.push_str("  ");
        }
    }

    line.trim_end_matches(' ').to_string()
}
