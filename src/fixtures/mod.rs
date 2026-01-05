use sqlx::PgConnection;

use crate::{AppError, AppState};

mod candidate_list;
mod persons;

pub async fn load(state: &AppState) -> Result<(), AppError> {
    let mut conn = state.pool().acquire().await?;

    clear_database(&mut conn).await?;
    persons::load(&mut conn).await?;
    candidate_list::load(&mut conn).await?;

    Ok(())
}

async fn clear_database(conn: &mut PgConnection) -> Result<(), AppError> {
    sqlx::query!(
        "
        TRUNCATE TABLE candidate_lists_persons, candidate_lists, persons
        RESTART IDENTITY CASCADE
        "
    )
    .execute(conn)
    .await?;

    Ok(())
}
