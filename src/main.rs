use yew::prelude::*;
use yewdux::prelude::*;
use web_sys::HtmlInputElement;
use berlewelch::*;

#[derive(Clone, PartialEq, Eq, Store)]
struct State {
    errors: i32,
    original: String,
    encoded: String,
    is_error: bool,
    // hack to force component rerendering to remove invalid characters from input elements even when no actual state was changed
    hack: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            errors: 2,
            original: String::new(),
            encoded: String::new(),
            is_error: false,
            hack: false,
        }
    }
}

fn clamp(number: i32, min: i32, max: i32) -> i32 {
    if number < min {
        min
    } else if number > max {
        max
    } else {
        number
    }
}

fn is_valid_message(msg: &str) -> bool {
    !msg.is_empty() && msg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ',' || c == '/')
}

fn str_to_c67(msg: &str) -> Vec<Gfe<67>> {
    msg.chars().map(|c| match c {
        '_' => 62,
        '-' => 63,
        '.' => 64,
        ',' => 65,
        '/' => 66,
        c if 'a' <= c && c <= 'z' => c as i64 - 'a' as i64,
        c if 'A' <= c && c <= 'Z' => c as i64 - 'A' as i64 + 26,
        c if '0' <= c && c <= '9' => c as i64 - '0' as i64 + 52,
        _ => panic!("Unexpected character"),
    }.into()).collect()
}

fn c67_to_str(msg: &[Gfe<67>]) -> String {
    msg.iter().copied().map(|x| match *x {
        62 => '_',
        63 => '-',
        64 => '.',
        65 => ',',
        66 => '/',
        x if x < 26 => char::from_u32(x + 'a' as u32).unwrap(),
        x if 26 <= x && x < 52 => char::from_u32(x - 26 + 'A' as u32).unwrap(),
        x if 52 <= x && x < 62 => char::from_u32(x - 52 + '0' as u32).unwrap(),
        _ => unreachable!(),
    }).collect()
}

fn my_encode(errors: usize, msg: &str) -> String {
    let c67 = str_to_c67(msg);
    let encoded = encode(errors, &c67);
    c67_to_str(&encoded)
}

fn my_decode(errors: usize, msg: &str) -> Result<String, ()> {
    let mut c67 = str_to_c67(msg);
    decode(errors, &mut c67)?;
    Ok(c67_to_str(&c67[..c67.len() - 2 * errors]))
}

#[function_component(App)]
fn app() -> Html {    
    html! {
        <div class="main-content">
            <h1>{ "Berlekamp-Welch Error Correction" }</h1>
            <h3>{ "About" }</h3>
            <p> {
                "This page demonstrates the Berlekamp-Welch algorithm. Messages are encoded into a form that is resistant to corruption
                errors. Partially-corrupted messages can be restored to their original values as long as no more than the specified maximum
                number of errors occur. The algorithm is relatively space-efficient, with the encoded message taking on
                only twice the number of maximum errors as additional characters on the end."
            } </p>
            <h3>{ "Instructions" }</h3>
            <p> {
                "Either of the message fields below can be edited. When the original message field is edited, the encoded message field updates to
                contain the error-resistant form of the message. When the encoded message field is edited, the original message field
                will update to contain what is believed to be the corresponding original message, or an error message will appear if the original
                message is known to be unrecoverable."
            } </p>
            <p>{ "Messages may only contain characters a-z, A-Z, digits, underscores, dashes, periods, commas, and slashes." }</p>
            <ErrorsInput />
            <InputOutput />
        </div>
    }
}

#[function_component(InputOutput)]
fn input_output() -> Html {
    let (state, dispatch) = use_store::<State>();
    
    let on_original_change = dispatch.reduce_mut_callback_with(|state, evt: InputEvent| {
        let target = evt.target_dyn_into::<HtmlInputElement>().unwrap();
        let new = target.value();
        if !new.is_empty() && !is_valid_message(&new) {
            state.hack = !state.hack;
            return;
        }
        let encoded = if new.is_empty() {
            String::new()
        } else {
            my_encode(state.errors as usize, &new)
        };
        state.original = new;
        state.encoded = encoded;
        state.is_error = false;
    });

    let on_encoded_change = dispatch.reduce_mut_callback_with(|state, evt: InputEvent| {
        let target = evt.target_dyn_into::<HtmlInputElement>().unwrap();
        let new = target.value();
        if !is_valid_message(&new) {
            state.hack = !state.hack;
            return;
        }
        if let Ok(decoded) = my_decode(state.errors as usize, &new) {
            state.original = decoded;
            state.is_error = false;
        } else {
            state.original = String::from("");
            state.is_error = true;
        }
        state.encoded = new;
    });

    html! {
        <div class="message-panel">
            <h4>{ "Original Message: " }</h4>
            { if state.is_error {
                html! { <input class="input" type="text" value="" placeholder="*error*" oninput={on_original_change}/> }
            } else {
                html! { <input class="input" type="text" value={ state.original.clone() } oninput={on_original_change}/> }
            } }
            <h4>{ "Encoded Message: " }</h4>
            <input class="input" type="text" value={ state.encoded.clone() } oninput={on_encoded_change}/>
            { if state.is_error { html! { <h4>{ "Decoding Error" }</h4> } } else { html! {} } }
        </div>
    }
}

#[function_component(ErrorsInput)]
fn errors_input() -> Html {
    let (state, dispatch) = use_store::<State>();
    
    let on_input = dispatch.reduce_mut_callback_with(|state, evt: Event| {
        let element = evt.target_dyn_into::<HtmlInputElement>().unwrap();
        state.errors = element.value().parse().ok().map(|x| clamp(x, 1, 50)).unwrap_or(state.errors);
        state.hack = !state.hack;
        
        if state.is_error {
            return;
        }

        let encoded = if state.original.is_empty() {
            String::new()
        } else {
            my_encode(state.errors as usize, &state.original)
        };
        state.encoded = encoded;
        state.is_error = false;
    });

    html! {
        <div class="errors-input">
            <label><h4>{ "Max Errors:" }</h4></label>
            <input type="number" value={ state.errors.to_string() } min="1" max="50" onchange={on_input} />
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}