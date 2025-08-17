use tower_lsp_server::lsp_types::{CompletionItemKind, Position, Range, SymbolKind};

pub fn koto_span_to_lsp_range(span: koto_parser::Span) -> Range {
    Range {
        start: koto_to_lsp_position(span.start),
        end: koto_to_lsp_position(span.end),
    }
}

pub fn koto_to_lsp_position(position: koto_parser::Position) -> Position {
    Position {
        line: position.line,
        character: position.column,
    }
}

pub fn default<T: Default>() -> T {
    T::default()
}

pub fn symbol_kind_to_completion_kind(symbol_kind: SymbolKind) -> CompletionItemKind {
    match symbol_kind {
        SymbolKind::FUNCTION => CompletionItemKind::FUNCTION,
        SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
        SymbolKind::FIELD => CompletionItemKind::FIELD,
        SymbolKind::NUMBER => CompletionItemKind::VALUE,
        SymbolKind::STRING => CompletionItemKind::VALUE,
        SymbolKind::BOOLEAN => CompletionItemKind::VALUE,
        SymbolKind::ARRAY => CompletionItemKind::VALUE,
        SymbolKind::OBJECT => CompletionItemKind::VALUE,
        SymbolKind::NULL => CompletionItemKind::VALUE,
        _ => CompletionItemKind::TEXT,
    }
}