use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use tokio::fs;

use crate::domains::notification::{EmailSender, domain::model::NotificationOutboxEntry};

#[derive(Debug, Clone)]
pub struct FileEmailSender {
    output_dir: PathBuf,
}

impl FileEmailSender {
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    fn file_path_for(&self, notification: &NotificationOutboxEntry) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%dT%H%M%S%3fZ");
        self.output_dir.join(format!(
            "{timestamp}_{}_{}.txt",
            notification.kind.as_str(),
            notification.notification_id
        ))
    }

    fn render_message(&self, notification: &NotificationOutboxEntry) -> String {
        format!(
            "To: {to}\nSubject: {subject}\nKind: {kind}\nNotification-Id: {id}\n\n{body}\n",
            to = notification.recipient_email,
            subject = notification.subject,
            kind = notification.kind.as_str(),
            id = notification.notification_id,
            body = notification.body_text
        )
    }
}

#[async_trait]
impl EmailSender for FileEmailSender {
    async fn send(&self, notification: &NotificationOutboxEntry) -> Result<(), String> {
        fs::create_dir_all(&self.output_dir)
            .await
            .map_err(|err| format!("failed to create notification output dir: {err}"))?;

        let path = self.file_path_for(notification);
        fs::write(&path, self.render_message(notification))
            .await
            .map_err(|err| format!("failed to write notification file {:?}: {err}", path))?;

        Ok(())
    }
}
