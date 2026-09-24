use std::fmt;

use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{layer, Layer};

pub struct ErrorDialogLayer<R: tauri::Runtime> {
    pub app: AppHandle<R>,
}

impl<S: Subscriber, R: tauri::Runtime> Layer<S> for ErrorDialogLayer<R> {
    fn on_event(&self, event: &Event<'_>, _ctx: layer::Context<'_, S>) {
        if *event.metadata().level() != tracing::Level::ERROR {
            return;
        }

        let mut visitor = MessageVisitor {
            message: String::new(),
        };

        event.record(&mut visitor);

        self.app
            .dialog()
            .message(visitor.message)
            .kind(MessageDialogKind::Error)
            .title("Error")
            .blocking_show();
    }
}

struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}
