use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;

use crate::db::{endpoints, snapshots};

pub async fn start_background_tasks(pool: PgPool, offline_threshold_minutes: i64) {
    // Start endpoint status updater
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        endpoint_status_updater(pool_clone, offline_threshold_minutes).await;
    });

    // Start snapshot cleanup (keep 7 days of data)
    tokio::spawn(async move {
        snapshot_cleanup(pool, 7).await;
    });
}

async fn endpoint_status_updater(pool: PgPool, threshold_minutes: i64) {
    let mut ticker = interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

        match endpoints::update_offline_endpoints(&pool, threshold_minutes).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Marked {} endpoints as offline", count);
                }
            }
            Err(e) => {
                tracing::error!("Error updating endpoint status: {:?}", e);
            }
        }
    }
}

async fn snapshot_cleanup(pool: PgPool, days_to_keep: i64) {
    let mut ticker = interval(Duration::from_secs(3600)); // Every hour

    loop {
        ticker.tick().await;

        match snapshots::cleanup_old_snapshots(&pool, days_to_keep).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Cleaned up {} old snapshots", count);
                }
            }
            Err(e) => {
                tracing::error!("Error cleaning up snapshots: {:?}", e);
            }
        }
    }
}
