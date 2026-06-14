use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::Modal;

pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Top,
    Bottom,
    ToggleSelect,
    CycleSort,
    Refresh,
    OpenConfirm,
    Confirm,
    CancelModal,
    DismissError,
    Nothing,
}

pub fn translate(modal: &Modal, key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::Nothing;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return Action::Quit;
    }

    match modal {
        Modal::Confirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::Confirm,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::CancelModal,
            _ => Action::Nothing,
        },
        Modal::Error(_) => Action::DismissError,
        Modal::Deleting => Action::Nothing,
        Modal::None => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Up | KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => Action::MoveDown,
            KeyCode::PageUp => Action::PageUp,
            KeyCode::PageDown => Action::PageDown,
            KeyCode::Home | KeyCode::Char('g') => Action::Top,
            KeyCode::End | KeyCode::Char('G') => Action::Bottom,
            KeyCode::Char(' ') => Action::ToggleSelect,
            KeyCode::Char('s') => Action::CycleSort,
            KeyCode::Char('r') => Action::Refresh,
            KeyCode::Char('d') => Action::OpenConfirm,
            _ => Action::Nothing,
        },
    }
}
