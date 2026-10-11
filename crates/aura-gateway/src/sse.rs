//! Incremental Server-Sent Events parser for upstream streams.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

#[derive(Default)]
pub struct SseParser {
    buf: Vec<u8>,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds bytes and returns complete events.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        self.buf.extend_from_slice(chunk);
        let mut out = Vec::new();
        while let Some((pos, delimiter)) = self.buf.iter().enumerate().find_map(|(pos, _)| {
            let tail = &self.buf[pos..];
            if tail.starts_with(b"\n\n") {
                Some((pos, 2))
            } else if tail.starts_with(b"\r\n\r\n") {
                Some((pos, 4))
            } else {
                None
            }
        }) {
            let block: Vec<u8> = self.buf.drain(..pos + delimiter).collect();
            if let Some(ev) = parse_block(&String::from_utf8_lossy(&block)) {
                out.push(ev);
            }
        }
        out
    }

    /// Flushes a trailing event without the final blank line.
    pub fn finish(&mut self) -> Option<SseEvent> {
        let rest = std::mem::take(&mut self.buf);
        parse_block(&String::from_utf8_lossy(&rest))
    }
}

fn parse_block(block: &str) -> Option<SseEvent> {
    let mut event = None;
    let mut data: Vec<&str> = Vec::new();
    for line in block.lines() {
        if line.starts_with(':') {
            continue;
        }
        let (field, value) = match line.split_once(':') {
            Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
            None => (line, ""),
        };
        match field {
            "event" => event = Some(value.to_string()),
            "data" => data.push(value),
            _ => {}
        }
    }
    if data.is_empty() && event.is_none() {
        return None;
    }
    Some(SseEvent {
        event,
        data: data.join("\n"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_codepoints_split_across_network_bytes_are_preserved() {
        let mut parser = SseParser::new();
        let mut events = Vec::new();
        for byte in "data: ação 🦋\r\n\r\n".as_bytes() {
            events.extend(parser.push(&[*byte]));
        }
        assert_eq!(
            events,
            vec![SseEvent {
                event: None,
                data: "ação 🦋".into()
            }]
        );
    }

    #[test]
    fn split_across_chunks_and_crlf() {
        let mut p = SseParser::new();
        assert!(p.push(b"data: {\"a\":").is_empty());
        let evs = p.push(b"1}\r\n\r\nevent: x\ndata: y\n\n: comment\n\n");
        assert_eq!(
            evs,
            vec![
                SseEvent {
                    event: None,
                    data: "{\"a\":1}".into()
                },
                SseEvent {
                    event: Some("x".into()),
                    data: "y".into()
                },
            ]
        );
        p.push(b"data: [DONE]");
        assert_eq!(p.finish().unwrap().data, "[DONE]");
    }
}
