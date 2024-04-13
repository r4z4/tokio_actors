pub struct Message {
	client_id: String,
	prefix: String,
	text: String,
}

pub struct WSMessage {
	text: String,
	headers: Headers
}

pub struct Message {
	text: String,
    client_id: i32,
}

pub struct Client {
	name: String,
}

pub struct Hub {
	id: sqlx::types::Uuid,
	name: String,
	clients: HashMap<Client, bool>,
	messages: Vec<Message>,
    // FIXME: broadcast vs. mpse
    bcast_send: broadcast::Sender<Message>,
    bcast_recv: broadcast::Receiver<Message>,
    register_send: broadcast::Sender<Client>,
	register_send: broadcast::Receiver<Client>,
    unregister_send: broadcast::Sender<Client>,
    unregister_send: broadcast::Receiver<Client>,
}

pub fn new_hub(name: String) -> Hub {
    let (bcast_send, bcast_recv) = broadcast::channel(8);
    let (register_send, register_recv) = broadcast::channel(8);
    let (unregister_send, unregister_recv) = broadcast::channel(8);
	Hub{
		id: Uuid.new_v4(),
		name: name,
		clients: HashMap<Client, bool>::new(),
        messages: vec![],
        bcast_send: bcast_send,
        bcast_recv: bcast_recv,
        register_send: register_send,
        register_recv: register_recv,
        unregister_send: unregister_send,
        unregister_recv: unregister_recv,
    }
}

// pub fn run(Extension(pool): Extension<PgPool>) {
// 	select {
// 		// These are our handleInfo callbacks
// 		case client := <-h.register:
// 			// Not concurrent. Add a lock if prod
// 			h.clients[client] = true
// 			log.Printf("client registered %s", client.id)

// 			// Get all existing messages when enter a room
// 			for _, msg := range h.messages {
// 				client.send <- getMessageTemplate(h.name, msg)
// 			}
// 		case client := <-h.unregister:
// 			if _, ok := h.clients[client]; ok {
// 				log.Printf("client unregistered %s", client.id)
// 				close(client.send)
// 				delete(h.clients, client)
// 			}
// 		case msg := <-h.broadcast:

// 			// Save to DB
// 			query := "INSERT INTO messages(sent_from, sent_to, message_text) VALUES(@sent_from, @sent_to, @message_text)"
// 			args := pgx.NamedArgs{"sent_from": 1, "sent_to": 1, "message_text": msg.Text}
// 			InsertQuery(dbPool, query, args)

// 			h.messages = append(h.messages, msg)

// 			for client := range h.clients {
// 				select {
// 				case client.send <- getMessageTemplate(h.name, msg):
// 				default:
// 					close(client.send)
// 					delete(h.clients, client)
// 				}
// 			}
// 		}
// 	}

pub struct MsgOptions {
	name: String,
	msg: &Message,
}

#[derive(Debug, Template, Deserialize)]
#[template(path = "chat_message.html")]
pub struct MessageTemplate {
    pub text: String,
}

pub fn get_message_template(name: String, msg: &Message) -> MessageTemplate {
    let text = "Hey there";
	MessageTemplate{ text: text}.into_response()
}
