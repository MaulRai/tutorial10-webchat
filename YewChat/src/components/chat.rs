use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

fn is_single_emoji(text: &str) -> bool {
    let trimmed = text.trim();
    let char_count = trimmed.chars().count();
    
    // Check if it's a single character and likely an emoji
    if char_count == 1 {
        let ch = trimmed.chars().next().unwrap();
        let code = ch as u32;
        // Common emoji ranges
        (code >= 0x1F600 && code <= 0x1F64F) || // Emoticons
        (code >= 0x1F300 && code <= 0x1F5FF) || // Misc Symbols
        (code >= 0x1F680 && code <= 0x1F6FF) || // Transport
        (code >= 0x1F700 && code <= 0x1F77F) || // Alchemical Symbols
        (code >= 0x2600 && code <= 0x26FF) ||   // Misc symbols
        (code >= 0x2700 && code <= 0x27BF) ||   // Dingbats
        (code >= 0xFE00 && code <= 0xFE0F) ||   // Variation Selectors
        (code >= 0x1F900 && code <= 0x1F9FF) || // Supplemental Symbols
        (code >= 0x1F018 && code <= 0x1F270)    // Various symbols
    } else {
        false
    }
}

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen h-screen bg-slate-50">
                // Sidebar with users
                <div class="flex-none w-64 h-full bg-white shadow-lg border-r border-gray-200">
                    <div class="bg-gradient-to-r from-blue-600 to-purple-600 text-white text-lg font-semibold p-4 shadow-sm">
                        {"👥 Online Users"}
                    </div>
                    <div class="p-3 space-y-2 overflow-y-auto h-full">
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div class="flex items-center p-3 hover:bg-gray-50 rounded-xl transition-colors duration-200 border border-gray-100 shadow-sm">
                                        <div class="relative">
                                            <img class="w-10 h-10 rounded-full border-2 border-green-400" src={u.avatar.clone()} alt="avatar"/>
                                            <div class="absolute -bottom-1 -right-1 w-4 h-4 bg-green-400 rounded-full border-2 border-white"></div>
                                        </div>
                                        <div class="ml-3 flex-grow">
                                            <div class="font-medium text-gray-800 text-sm">
                                                {u.name.clone()}
                                            </div>
                                            <div class="text-xs text-green-500 font-medium">
                                                {"Online"}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>
                
                // Main chat area
                <div class="flex-1 h-full flex flex-col">
                    // Header
                    <div class="bg-white border-b border-gray-200 shadow-sm">
                        <div class="flex items-center p-4">
                            <div class="text-xl font-bold text-gray-800">{"💬 Chat Room"}</div>
                            <div class="ml-auto">
                                <div class="flex items-center space-x-2 text-sm text-gray-500">
                                    <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                                    <span>{format!("{} users online", self.users.len())}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    // Messages area
                    <div class="flex-1 overflow-y-auto p-4 space-y-4 bg-gradient-to-b from-slate-50 to-blue-50">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="flex items-start space-x-3 max-w-2xl">
                                        <img class="w-8 h-8 rounded-full border-2 border-white shadow-md flex-shrink-0" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="bg-white rounded-2xl rounded-tl-sm shadow-md border border-gray-100 p-4 flex-1">
                                            <div class="flex items-center space-x-2 mb-1">
                                                <span class="font-semibold text-gray-800 text-sm">{m.from.clone()}</span>
                                                <span class="text-xs text-gray-400">{"just now"}</span>
                                            </div>
                                            <div class="text-gray-700">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-2 rounded-lg max-w-xs shadow-sm" src={m.message.clone()}/>
                                                } else if is_single_emoji(&m.message) {
                                                    <p class="text-6xl leading-relaxed">{m.message.clone()}</p>
                                                } else {
                                                    <p class="leading-relaxed">{m.message.clone()}</p>
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    
                    // Input area
                    <div class="bg-white border-t border-gray-200 p-4 shadow-lg">
                        <div class="flex items-center space-x-3 max-w-4xl mx-auto">
                            <div class="flex-1 relative">
                                <input 
                                    ref={self.chat_input.clone()} 
                                    type="text" 
                                    placeholder="Type your message..." 
                                    class="w-full py-3 px-4 pr-12 bg-gray-100 border border-gray-200 rounded-full outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200" 
                                    name="message" 
                                    required=true 
                                />
                            </div>
                            <button 
                                onclick={submit} 
                                class="p-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 w-12 h-12 rounded-full flex justify-center items-center shadow-lg hover:shadow-xl transition-all duration-200 transform hover:scale-105"
                            >
                                <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" class="w-5 h-5 text-white">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}