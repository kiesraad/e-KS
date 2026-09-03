//! A stub BRP for tests, so the BRP checks can be exercised without the
//! `personen-mock` container and `cargo test` stays hermetic.

use std::{net::SocketAddr, sync::Arc};

use axum::{Json, Router, extract::State, routing::post};
use parking_lot::Mutex;
use serde_json::{Value, json};
use tokio::{net::TcpListener, task::JoinHandle};

use crate::{constants, structs::brp::BrpClient};

#[derive(Clone)]
struct StubState {
    persons: Arc<Value>,
    queries: Arc<Mutex<Vec<Value>>>,
}

/// A BRP that answers from a canned list of `personen` and records the queries
/// it was sent.
///
/// The stub answers every request with the whole list; matching a candidate to
/// their record by burgerservicenummer is the client's job.
pub struct BrpStub {
    pub client: BrpClient,
    queries: Arc<Mutex<Vec<Value>>>,
    server: JoinHandle<()>,
}

impl BrpStub {
    pub async fn serving(persons: Vec<Value>) -> Self {
        let queries = Arc::new(Mutex::new(Vec::new()));
        let state = StubState {
            persons: Arc::new(json!({
                "type": "RaadpleegMetBurgerservicenummer",
                "personen": persons,
            })),
            queries: Arc::clone(&queries),
        };

        let router = Router::new()
            .route(
                &format!("/{}", constants::BRP_PERSONS_ENDPOINT),
                post(
                    |State(state): State<StubState>, Json(query): Json<Value>| async move {
                        state.queries.lock().push(query);
                        Json((*state.persons).clone())
                    },
                ),
            )
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self {
            client: BrpClient::new_for_test(&format!("http://{addr}")),
            queries,
            server,
        }
    }

    /// How many requests the stub has been sent.
    pub fn query_count(&self) -> usize {
        self.queries.lock().len()
    }

    /// The single query the stub was sent.
    pub fn only_query(&self) -> Value {
        let queries = self.queries.lock();
        assert_eq!(queries.len(), 1, "expected exactly one BRP request");
        queries[0].clone()
    }
}

impl Drop for BrpStub {
    fn drop(&mut self) {
        self.server.abort();
    }
}

/// A BRP record that matches [`crate::test_utils::sample_person_from_brp`] on
/// every checked field, so a test only has to change the one thing it is about.
pub fn matching_record(bsn: &str) -> Value {
    json!({
        "burgerservicenummer": bsn,
        "naam": {
            "geslachtsnaam": "Bruin",
            "voorvoegsel": "de",
            "voorletters": "T.",
        },
        "geslacht": { "code": "V" },
        "geboorte": { "datum": { "datum": "1990-12-11" } },
        "nationaliteiten": [ { "nationaliteit": { "code": "0001" } } ],
        "uitsluitingKiesrecht": { "uitgeslotenVanKiesrecht": false },
        "verblijfplaats": {
            "type": "Adres",
            "verblijfadres": { "woonplaats": "Utrecht" },
        },
    })
}
