use {
    super::{Link, Tree},
    colored::Colorize,
    std::fmt::{Debug, Formatter, Result},
};

impl Debug for Tree {
    fn fmt(&self, f: &mut Formatter) -> Result {
        fn traverse(
            f: &mut Formatter,
            cursor: &Tree,
            stack: &mut Vec<(Vec<u8>, Vec<u8>)>,
            left: bool,
        ) -> Result {
            if let Some(child_link) = cursor.link(true) {
                stack.push((child_link.key().to_vec(), cursor.key().to_vec()));
                if let Some(child_tree) = child_link.tree() {
                    traverse(f, child_tree, stack, true)
                } else {
                    traverse_pruned(f, child_link, stack, true)
                }?;
                stack.pop();
            }

            let depth = stack.len();

            if depth > 0 {
                // draw ancestor's vertical lines
                for (low, high) in stack.iter().take(depth - 1) {
                    let draw_line = cursor.key() > low && cursor.key() < high;
                    write!(f, "{}", if draw_line { " │  " } else { "    " }.dimmed())?;
                }
            }

            let prefix = if depth == 0 {
                ""
            } else if left {
                " ┌-"
            } else {
                " └-"
            };
            writeln!(
                f,
                "{}{}",
                prefix.dimmed(),
                format!("{:?}", cursor.key()).on_bright_black()
            )?;

            if let Some(child_link) = cursor.link(false) {
                stack.push((cursor.key().to_vec(), child_link.key().to_vec()));
                if let Some(child_tree) = child_link.tree() {
                    traverse(f, child_tree, stack, false)
                } else {
                    traverse_pruned(f, child_link, stack, false)
                }?;
                stack.pop();
            }
            Ok(())
        }

        fn traverse_pruned(
            f: &mut Formatter,
            link: &Link,
            stack: &mut Vec<(Vec<u8>, Vec<u8>)>,
            left: bool,
        ) -> Result {
            let depth = stack.len();

            if depth > 0 {
                // draw ancestor's vertical lines
                for (low, high) in stack.iter().take(depth - 1) {
                    let draw_line = link.key() > low && link.key() < high;
                    write!(f, "{}", if draw_line { " │  " } else { "    " }.dimmed()).unwrap();
                }
            }

            let prefix = if depth == 0 {
                ""
            } else if left {
                " ┌-"
            } else {
                " └-"
            };
            writeln!(
                f,
                "{}{}",
                prefix.dimmed(),
                format!("{:?}", link.key()).blue()
            )
        }

        let mut stack = vec![];
        traverse(f, self, &mut stack, false)?;
        writeln!(f)
    }
}
