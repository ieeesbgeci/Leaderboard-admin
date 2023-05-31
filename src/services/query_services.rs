use std::marker::PhantomData;

use actix::{
    Actor, AsyncContext, ContextFutureSpawner, Handler, Message, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};
use uuid::Uuid;

use crate::models::error_models::VaderError;
use crate::models::query_models::{EventInfo, EventQuery, EventType, FtsQuery, TeamInfo};
use crate::models::v_models::{AsyncDbRes, Event, EventState, Player, Team, User};
impl FromRow<'_, SqliteRow> for Team<'_> {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = Uuid::parse_str(row.get("id")).map_err(|_e| sqlx::Error::ColumnDecode {
            index: "0".to_string(),
            source: Box::new(VaderError::SqlxFieldError("Error decoding Team Id")),
        })?;
        let name: String = row.get("name");
        let score: i64 = row.get("score");
        let logo: Option<String> = row.get("logo");
        let tm: String = row.get("team_members");
        let members: Vec<Uuid> = tm
            .split(',')
            .collect::<Vec<&str>>()
            .iter()
            .map(|m| {
                Uuid::parse_str(m).map_err(|_e| sqlx::Error::ColumnDecode {
                    index: "0".to_string(),
                    source: Box::new(VaderError::SqlxFieldError("Error decoding Event Id")),
                })
            })
            .collect::<Result<Vec<Uuid>, sqlx::Error>>()?;
        Ok(Team {
            id,
            name: name.into(),
            logo,
            score,
            members,
        })
    }
}
impl FromRow<'_, SqliteRow> for User<'_> {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = Uuid::parse_str(row.get("id")).map_err(|_e| sqlx::Error::ColumnDecode {
            index: "0".to_string(),
            source: Box::new(VaderError::SqlxFieldError("Error decoding User Id")),
        })?;
        let name: String = row.get("name");
        let score: i64 = row.get("score");
        let logo: Option<String> = row.get("logo");
        Ok(User {
            id,
            name: name.into(),
            logo,
            score,
        })
    }
}

impl FromRow<'_, SqliteRow> for TeamInfo<'_> {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = Uuid::parse_str(row.get("id")).map_err(|_e| sqlx::Error::ColumnDecode {
            index: "0".to_string(),
            source: Box::new(VaderError::SqlxFieldError("Error decoding User Id")),
        })?;
        let name: String = row.get("name");
        let score: i64 = row.get("score");
        let logo: Option<String> = row.get("logo");
        Ok(TeamInfo {
            id,
            name: name.into(),
            logo,
            score,
        })
    }
}

impl<'a, T, U> FromRow<'_, SqliteRow> for Event<'a, T, U>
where
    T: Player<'a>,
    U: EventState,
{
    fn from_row(row: &'_ SqliteRow) -> Result<Event<'a, T, U>, sqlx::Error> {
        let id_str: String = row.get("id");
        let id: Uuid = Uuid::parse_str(&id_str).map_err(|_e| sqlx::Error::ColumnDecode {
            index: "0".to_string(),
            source: Box::new(VaderError::SqlxFieldError("Error decoding Event Id")),
        })?;

        let name: String = row.get("name");
        let logo: Option<String> = row.get("logo");

        Ok(Event {
            id,
            name: name.into(),
            logo,
            player_marker: PhantomData::<&'a T>,
            state_marker: PhantomData::<&'a U>,
        })
    }
}
impl<'a, U> Event<'a, Team<'a>, U>
where
    U: EventState,
{
    pub fn get_info(&'a self, db_pool: &'a SqlitePool) -> AsyncDbRes<'a, EventQuery<Team>> {
        let event_id = self.id.to_string();
        Box::pin(async move {
            let event = sqlx::query_as::<_, Event<'a, Team, U>>(
                "SELECT id,name,logo FROM events WHERE id = ?",
            )
            .bind(&event_id)
            .fetch_one(db_pool)
            .await?;
            let contestants = sqlx::query_as::<_, Team>("SELECT t.id AS id,t.name AS name,t.score AS score,t.logo AS logo,GROUP_CONCAT(tm.user_id,',') AS team_members FROM events e JOIN event_teams et ON et.event_id = e.id JOIN teams t ON et.team_id=t.id JOIN team_members tm ON tm.team_id = t.id WHERE e.id = ? GROUP BY t.id")
                .bind(&event_id)
                .fetch_all(db_pool)
                .await?;

            Ok(EventQuery {
                id: event.id,
                name: event.name,
                logo: event.logo,
                contestants,
                event_type: EventType::TeamEvent,
                marker: PhantomData::<&'a Team>,
            })
        })
    }
}
impl<'a, U> Event<'a, User<'a>, U>
where
    U: EventState,
{
    pub fn get_info(&'a self, db_pool: &'a SqlitePool) -> AsyncDbRes<'a, EventQuery<User>> {
        let event_id = self.id.to_string();
        Box::pin(async move {
            let event = sqlx::query_as::<_, Event<'a, User, U>>(
                "SELECT id,name,logo FROM events WHERE id = ?",
            )
            .bind(&event_id)
            .fetch_one(db_pool)
            .await?;
            let contestants = sqlx::query_as::<_, User>(
                "SELECT u.name AS name,u.score AS score,u.logo AS logo FROM events e JOIN event_users eu on eu.event_id=e.id JOIN users u on eu.user_id=u.id where e.id = ? GROUP BY u.id",
            )
            .bind(&event_id)
            .fetch_all(db_pool)
            .await?;

            Ok(EventQuery {
                id: event.id,
                name: event.name,
                logo: event.logo,
                contestants,
                event_type: EventType::UserEvent,
                marker: PhantomData::<&'a User>,
            })
        })
    }
}
impl<'a, 'b> FromRow<'a, SqliteRow> for EventInfo<'b> {
    fn from_row(row: &'a SqliteRow) -> Result<EventInfo<'b>, sqlx::Error> {
        let id_str: String = row.get("id");
        let id: Uuid = Uuid::parse_str(&id_str).map_err(|_e| sqlx::Error::ColumnDecode {
            index: "0".to_string(),
            source: Box::new(VaderError::SqlxFieldError("Error decoding Event Id")),
        })?;
        let name: String = row.get("name");
        let logo: Option<String> = row.get("logo");
        let type_str: String = row.get("event_type");
        let event_type = match type_str.as_str() {
            "team_event" => EventType::TeamEvent,
            "user_event" => EventType::UserEvent,
            _ => {
                return Err(sqlx::Error::ColumnDecode {
                    index: "0".to_string(),
                    source: Box::new(VaderError::SqlxFieldError("Error decoding EventType")),
                });
            }
        };

        Ok(EventInfo {
            id,
            name: name.into(),
            logo,
            event_type,
        })
    }
}

impl EventInfo<'_> {
    pub fn get_event_info<'a>(event_id: &'a Uuid, db_pool: &'a SqlitePool) -> AsyncDbRes<'a, Self> {
        let id = event_id.to_string();
        Box::pin(async move {
            let event = sqlx::query_as::<_, EventInfo>(
                "SELECT id,name,logo,event_type from events WHERE id = ?",
            )
            .bind(&id)
            .fetch_one(db_pool)
            .await?;
            Ok(event)
        })
    }
}

pub trait Queriable {
    type QueryRes;
    fn fts_query<'a, 'b>(
        param: &'a str,
        db_pool: &'b SqlitePool,
    ) -> AsyncDbRes<'a, Vec<Self::QueryRes>>
    where
        'b: 'a;
}
impl Queriable for TeamInfo<'_> {
    type QueryRes = Self;
    fn fts_query<'a, 'b>(
        param: &'a str,
        db_pool: &'b SqlitePool,
    ) -> AsyncDbRes<'a, Vec<Self::QueryRes>>
    where
        'b: 'a,
    {
        Box::pin(async move {
            let teams = sqlx::query_as::<_, TeamInfo>(
                "SELECT id,name,score,logo FROM teams_fts WHERE name MATCH  ? ",
            )
            .bind(format!("{}*", param))
            .fetch_all(db_pool)
            .await?;
            Ok(teams)
        })
    }
}
impl Queriable for User<'_> {
    type QueryRes = Self;
    fn fts_query<'a, 'b>(
        param: &'a str,
        db_pool: &'b SqlitePool,
    ) -> AsyncDbRes<'a, Vec<Self::QueryRes>>
    where
        'b: 'a,
    {
        Box::pin(async move {
            let users = sqlx::query_as::<_, User>(
                "SELECT id,name,score,logo FROM users_fts WHERE name MATCH  ? ",
            )
            .bind(format!("{}*", param))
            .fetch_all(db_pool)
            .await?;
            Ok(users)
        })
    }
}
impl Queriable for EventInfo<'_> {
    type QueryRes = Self;
    fn fts_query<'a, 'b>(
        param: &'a str,
        db_pool: &'b SqlitePool,
    ) -> AsyncDbRes<'a, Vec<Self::QueryRes>>
    where
        'b: 'a,
    {
        Box::pin(async move {
            let events = sqlx::query_as::<_, EventInfo>(
                "SELECT id,name,logo,event_type FROM events_fts WHERE name MATCH  ? ",
            )
            .bind(format!("{}*", param))
            .fetch_all(db_pool)
            .await?;
            Ok(events)
        })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct FtsQueryRes(String);

impl<'a> Actor for FtsQuery<'a, TeamInfo<'_>>
where
    'a: 'static,
{
    type Context = ws::WebsocketContext<Self>;
}

type WsMsg = Result<ws::Message, ws::ProtocolError>;

impl<'a> Handler<FtsQueryRes> for FtsQuery<'a, TeamInfo<'_>>
where
    'a: 'static,
{
    type Result = ();
    fn handle(&mut self, msg: FtsQueryRes, ctx: &mut Self::Context) {
        let res_str = msg.0;
        ctx.text(res_str);
    }
}

impl<'a> StreamHandler<WsMsg> for FtsQuery<'a, TeamInfo<'_>>
where
    'a: 'static,
{
    fn handle(&mut self, msg: WsMsg, ctx: &mut Self::Context) {
        use ws::Message::*;
        let pool = self.db_pool.clone();
        let addr = ctx.address();
        match msg {
            Ok(Ping(msg)) => ctx.pong(&msg),
            Ok(Text(param)) => {
                async move {
                    let teams = TeamInfo::fts_query(&param, &pool)
                        .await
                        .ok()
                        .and_then(|t| serde_json::to_string(&t).ok());
                    if let Some(teams_str) = teams {
                        addr.do_send(FtsQueryRes(teams_str))
                    }
                }
                .into_actor(self)
                .wait(ctx);
            }
            _ => (),
        }
    }
}

impl<'a> Actor for FtsQuery<'a, User<'_>>
where
    'a: 'static,
{
    type Context = ws::WebsocketContext<Self>;
}

impl<'a> Handler<FtsQueryRes> for FtsQuery<'a, User<'_>>
where
    'a: 'static,
{
    type Result = ();
    fn handle(&mut self, msg: FtsQueryRes, ctx: &mut Self::Context) {
        let res_str = msg.0;
        ctx.text(res_str);
    }
}

impl<'a> StreamHandler<WsMsg> for FtsQuery<'a, User<'_>>
where
    'a: 'static,
{
    fn handle(&mut self, msg: WsMsg, ctx: &mut Self::Context) {
        use ws::Message::*;
        let pool = self.db_pool.clone();
        let addr = ctx.address();
        match msg {
            Ok(Ping(msg)) => ctx.pong(&msg),
            Ok(Text(param)) => {
                async move {
                    let users = User::fts_query(&param, &pool)
                        .await
                        .ok()
                        .and_then(|t| serde_json::to_string(&t).ok());
                    if let Some(users_str) = users {
                        addr.do_send(FtsQueryRes(users_str))
                    }
                }
                .into_actor(self)
                .wait(ctx);
            }
            _ => (),
        }
    }
}

impl<'a> Actor for FtsQuery<'a, EventInfo<'_>>
where
    'a: 'static,
{
    type Context = ws::WebsocketContext<Self>;
}

impl<'a> Handler<FtsQueryRes> for FtsQuery<'a, EventInfo<'_>>
where
    'a: 'static,
{
    type Result = ();
    fn handle(&mut self, msg: FtsQueryRes, ctx: &mut Self::Context) {
        let res_str = msg.0;
        ctx.text(res_str);
    }
}

impl<'a> StreamHandler<WsMsg> for FtsQuery<'a, EventInfo<'_>>
where
    'a: 'static,
{
    fn handle(&mut self, msg: WsMsg, ctx: &mut Self::Context) {
        use ws::Message::*;
        let pool = self.db_pool.clone();
        let addr = ctx.address();
        match msg {
            Ok(Ping(msg)) => ctx.pong(&msg),
            Ok(Text(param)) => {
                async move {
                    let events = EventInfo::fts_query(&param, &pool)
                        .await
                        .ok()
                        .and_then(|t| serde_json::to_string(&t).ok());
                    if let Some(events_str) = events {
                        addr.do_send(FtsQueryRes(events_str))
                    }
                }
                .into_actor(self)
                .wait(ctx);
            }
            _ => (),
        }
    }
}
