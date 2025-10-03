use rocket::serde::Serialize;
use rocket::serde::uuid::Uuid;
use rocket::tokio::sync::mpsc;
use sqlx::{Pool, Postgres};
use std::sync::{Arc, Mutex};
use tracing::Event;
use tracing::field::{Field, Visit};
use tracing_subscriber::Layer;

// The type of data we send over the channel
#[derive(Debug)]
pub struct LogMessage {
    span_id: Uuid,
    tool_use: bool,
    message: String,
}

// The async worker context
pub struct AsyncDbWorker {
    // The receiver half of the channel, owned by the worker task
    pub rx: mpsc::Receiver<LogMessage>,
    pub db_client: Pool<Postgres>,
}

// The synchronous Layer that feeds the worker
pub struct PSqlLayer {
    // The sender half of the channel, wrapped in a thread-safe container
    pub tx: Arc<Mutex<mpsc::Sender<LogMessage>>>,
}

impl<S: tracing::Subscriber> Layer<S> for PSqlLayer {
    fn on_event(&self, event: &Event, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = PsqlVisitor::new();

        // Tell the event to record all its fields into our visitor
        event.record(&mut visitor);

        // 1. Extract data synchronously
        let log_data = LogMessage {
            span_id: Uuid::parse_str(
                &visitor
                    .span_id
                    .unwrap_or_else(|| "span needs to exist".to_string()),
            )
            .unwrap_or_else(|_e| Uuid::new_v4()),
            tool_use: visitor.tool_use.unwrap_or_else(|| false),
            message: visitor.message.unwrap_or_else(|| "no message".to_string()),
        };

        // 2. Send data to the background worker (this is non-blocking)
        if let Ok(sender) = self.tx.lock() {
            let _ = sender.try_send(log_data);
        }
    }
    // ... other synchronous trait methods
}

pub async fn run_async_worker(mut worker: AsyncDbWorker) -> anyhow::Result<()> {
    while let Some(log_message) = worker.rx.recv().await {
        let _ = sqlx::query!(
            r#"
            INSERT INTO traces (span_id, tool_use, message, timestamp)
            VALUES ($1, $2, $3, NOW())
            "#,
            &log_message.span_id,
            &log_message.tool_use,
            &log_message.message
        )
        .execute(&worker.db_client)
        .await?;
    }
    Ok(())
}

struct PsqlVisitor {
    /// The main log message, usually found under the "message" field.
    message: Option<String>,
    /// Stores the captured log level.
    tool_use: Option<bool>,
    span_id: Option<String>,
    /// Stores any custom fields found.
    custom_fields: Vec<(String, String)>,
}

impl PsqlVisitor {
    fn new() -> Self {
        PsqlVisitor {
            message: None,
            span_id: None,
            tool_use: None,
            custom_fields: Vec::new(),
        }
    }
}

/// Implement the Visit trait to define how different field types are extracted.
impl Visit for PsqlVisitor {
    /// Handler for string fields, primarily used to capture the main "message".
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "message" => {
                self.message = Some(value.to_string());
            }
            "span_id" => {
                self.span_id = Some(value.to_string());
            }
            _ => {
                self.custom_fields
                    .push((field.name().to_string(), value.to_string()));
            }
        }
    }

    /// Handler for debug-printable fields (the default fallback).
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // We use this for all types other than string/integers we specifically care about.
        if field.name() == "message" {
            // Even if the message isn't a string literal, try to capture it.
            self.message = Some(format!("{:?}", value));
        } else {
            // Capture custom fields as debug strings
            self.custom_fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "tool_use" {
            self.tool_use = Some(value);
        } else {
            self.custom_fields
                .push((field.name().to_string(), value.to_string()));
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SpanLength {
    range: String,
    frequency: i64,
}
pub async fn get_histogram(pool: &Pool<Postgres>) -> anyhow::Result<Vec<SpanLength>> {
    let spans = sqlx::query_as!(
        SpanLength,
        r#"
            SELECT
            int4range(
                30*(buckets-1),
                30*buckets,
                '[]'
            )::TEXT as "range!: String",
            frequency as "frequency!" FROM
            (
                SELECT width_bucket(diff_in_seconds, 0, 180, 8) as buckets,
                count(span_id) as frequency
                FROM
                (
                    SELECT COALESCE(EXTRACT(EPOCH FROM (MAX(timestamp) - MIN(timestamp))), 0)::double precision as diff_in_seconds,
                    span_id from
                    traces
                    where timestamp> date_subtract(NOW(), '7 day'::interval)
                    group by span_id
                ) group by buckets
            ) t
            ORDER BY buckets asc
            "#
    )
    .fetch_all(pool)
    .await?;
    Ok(spans)
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SpanToolUse {
    cnt_spns_with_tools: i64,
    cnt_spns_without_tools: i64,
    date: chrono::DateTime<chrono::Utc>,
}
pub async fn get_tool_use(pool: &Pool<Postgres>) -> anyhow::Result<Vec<SpanToolUse>> {
    let spans = sqlx::query_as!(
        SpanToolUse,
        r#"
            SELECT
            SUM(CASE WHEN used_tools then 1 else 0 end) as "cnt_spns_with_tools!",
            SUM(CASE WHEN not used_tools then 1 else 0 end) as "cnt_spns_without_tools!",
            date as "date!" FROM
            (
                SELECT CASE WHEN
                    MAX(CASE WHEN tool_use is true then 1 else 0 END)=1
                    then true else false END as used_tools,
                    date_trunc('day', timestamp) as date,
                span_id from
                traces
                where timestamp> date_subtract(NOW(), '7 day'::interval)
                group by span_id, date_trunc('day', timestamp)
            )
            group by date
            "#
    ) //-- where timestamp> NOW()::DATE-EXTRACT(DOW FROM NOW())::INTEGER-7
    .fetch_all(pool)
    .await?;
    Ok(spans)
}
