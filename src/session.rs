use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server::{self, ClientMessage};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsCalSession {
    /// unique session id
    pub id: usize,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// Cal server
    pub addr: Addr<server::CalServer>,
}

impl WsCalSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

    fn list_rooms(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        self.addr
            .send(server::ListCals)
            .into_actor(self)
            .then(|res, _act, ctx| {
                ctx.text(res.unwrap());
                fut::ready(())
            })
            .wait(ctx)
    }

    fn create_cal(&mut self, msg: server::CreateCal, ctx: &mut ws::WebsocketContext<Self>) {
        self.addr
            .send(msg)
            .into_actor(self)
            .then(|res, _act, ctx| {
                ctx.text(match res {
                    Ok(v) => match v {
                        Ok(s) => s,
                        Err(e) => e.to_string(),
                    },
                    Err(e) => e.to_string(),
                });

                fut::ready(())
            })
            .wait(ctx)
    }

    fn join_cal(&mut self, msg: server::Join, ctx: &mut ws::WebsocketContext<Self>) {
        self.addr
            .send(msg)
            .into_actor(self)
            .then(|res, _act, ctx| {
                ctx.text(match res {
                    Ok(v) => match v {
                        Ok(s) => s,
                        Err(e) => e.to_string(),
                    },
                    Err(e) => e.to_string(),
                });

                fut::ready(())
            })
            .wait(ctx)
    }

    fn add_event(&mut self, msg: server::AddEvent, ctx: &mut ws::WebsocketContext<Self>) {
        self.addr
            .send(msg)
            .into_actor(self)
            .then(|res, _act, ctx| {
                ctx.text(match res {
                    Ok(v) => match v {
                        Ok(s) => format!("EventID: {:?}", s),
                        Err(e) => e.to_string(),
                    },
                    Err(e) => e.to_string(),
                });

                fut::ready(())
            })
            .wait(ctx)
    }

    fn del_event(&mut self, msg: server::DeleteEvent, ctx: &mut ws::WebsocketContext<Self>) {
        self.addr
            .send(msg)
            .into_actor(self)
            .then(|res, _act, ctx| {
                ctx.text(match res {
                    Ok(inner) => match inner {
                        Err(e) => e.to_string(),
                        _ => "Event Deleted".to_string(),
                    },
                    Err(e) => e.to_string(),
                });

                fut::ready(())
            })
            .wait(ctx)
    }

    fn get_event(&mut self, msg: server::GetEvent, ctx: &mut ws::WebsocketContext<Self>) {
        self.addr
            .send(msg)
            .into_actor(self)
            .then(|res, _act, ctx| {
                ctx.text(match res {
                    Ok(inner) => match inner {
                        Ok(s) => s,
                        Err(e) => e.to_string(),
                    },
                    Err(e) => e.to_string(),
                });

                fut::ready(())
            })
            .wait(ctx)
    }

    fn get_event_range(
        &mut self,
        msg: server::GetEventsInRange,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        self.addr
            .send(msg)
            .into_actor(self)
            .then(|res, _act, ctx| {
                ctx.text(match res {
                    Ok(inner) => match inner {
                        Ok(s) => s,
                        Err(e) => e.to_string(),
                    },
                    Err(e) => e.to_string(),
                });

                fut::ready(())
            })
            .wait(ctx)
    }
}

// WsChatSession is a "middle man" between the server and the client.
impl Actor for WsCalSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Session started");
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        // send Connect message to ChatServer
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsCalSession {
    type Result = ();

    /// if we recieve a server::Message from ChatServer then forward it over to the client
    fn handle(&mut self, _msg: server::Message, _ctx: &mut Self::Context) {}
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsCalSession {
    // handle incoming messages from the client
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let msg: server::ClientMessage = match serde_json::from_slice(text.as_bytes()) {
                    Ok(m) => m,
                    Err(e) => {
                        ctx.text(format!(
                            "The message recieved was not understood by the server: {} ",
                            e.to_string()
                        ));
                        return;
                    }
                };

                // the json sent to the server has been successfully parsed

                match msg {
                    ClientMessage::ListCals => self.list_rooms(ctx),
                    ClientMessage::Join(join_msg) => self.join_cal(join_msg, ctx),
                    ClientMessage::CreateCal(create_msg) => self.create_cal(create_msg, ctx),
                    ClientMessage::AddEvent(add_msg) => self.add_event(add_msg, ctx),
                    ClientMessage::DeleteEvent(del_msg) => self.del_event(del_msg, ctx),
                    ClientMessage::GetEvent(get_event_msg) => self.get_event(get_event_msg, ctx),
                    ClientMessage::GetEventsInRange(get_range_msg) => {
                        self.get_event_range(get_range_msg, ctx)
                    }
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
