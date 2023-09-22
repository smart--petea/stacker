use crate::forms;
use crate::models;
use crate::startup::AppState;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::instrument;
use tracing::Instrument;
use uuid::Uuid;

// workflow
// add, update, list, get(user_id), ACL,
// ACL - access to func for a user
// ACL - access to objects for a user

pub async fn rating(
    app_state: web::Data<AppState>,
    form: web::Json<forms::Rating>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    //TODO. check if there already exists a rating for this product committed by this user
    let request_id = Uuid::new_v4();
    match sqlx::query_as!(
        models::Product,
        r"SELECT * FROM product WHERE obj_id = $1",
        form.obj_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(product) => {
            tracing::info!("req_id: {} Found product: {:?}", request_id, product.obj_id);
        }
        Err(e) => {
            tracing::error!(
                "req_id: {} Failed to fetch product: {:?}, error: {:?}",
                request_id,
                form.obj_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_id = app_state.user_id; // uuid Let's assume we have a user id already taken from auth
    let query_span = tracing::info_span!("Saving new rating details in the database");
    // Get product by id
    // Insert rating
    //let category = Into::<String>::into(form.category.clone());
    match sqlx::query!(
        r#"
        INSERT INTO rating (user_id, product_id, category, comment, hidden,rate,
        created_at,
        updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, NOW() at time zone 'utc', NOW() at time zone 'utc')
        "#,
        user_id,
        form.obj_id,
        form.category as models::RateCategory,
        form.comment,
        false,
        form.rate
    )
    .execute(pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "req_id: {} New subscriber details have been saved to database",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("req_id: {} Failed to execute query: {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
