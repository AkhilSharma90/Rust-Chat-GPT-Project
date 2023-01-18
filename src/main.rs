use dotenv::dotenv;
use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env; // env module for env variables, OpenAi access key
use std::io::{stdin, stdout, Write};

#[derive(Deserialize, Debug)]
struct OAIChoices {
    text: String,
    index: u8,
    logprobs: Option<u8>,
    finish_reason: String,
} // Subset of the OpenAI API response

#[derive(Deserialize, Debug)]
struct OAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OAIChoices>, // Vector of OAIChoices struct
} // Making most optional, just so that its defensive

#[derive(Serialize, Debug)]
struct OAIRequest {
    prompt: String, // question we are asking
    max_tokens: u16, // max tokens you want in your response
                    // temperature: f32,
                    // top_p: f32,
                    // frequency_penalty: f32,
                    // presence_penalty: f32,
                    // stop: Vec<String>,
} // Struct for the OpenAI API request

#[tokio::main] // using async functions
               // Box type to handle errors
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok(); // load env variables from .env file
    let https = HttpsConnector::new(); // https connector from hyper_tls crate
    let client = Client::builder().build(https); // client from hyper crate
    let uri = "https://api.openai.com/v1/engines/text-davinci-001/completions"; // OpenAI API endpoint

    let preamble = "Generate a Sql code for the given statement."; // preamble for the prompt


    let oai_token: String = env::var("OAI_TOKEN").unwrap(); // getting the OAI_TOKEN from the environment variables
                                                            // If some, unwrap, else (none) panic
    let auth_header_val = format!("Bearer {}", oai_token); // formatting the auth header value, prepend Bearer String with the OAI_TOKEN
                                                           // expects the value to be Bearer _ whatever the token is

    println!("{esc}c", esc = 27 as char);

    loop {
        print!("> "); // println flushes it, print doesn't
        stdout().flush().unwrap(); // flush stdout
        let mut user_text = String::new(); // variable to store user input

        // When you write data to a stream, it is not written immediately, and it is buffered. Use flush() when you need to be sure that all your data from buffer is written

        // Flushing a stream ensures that all data that has been written to that stream is output, including clearing any that may have been buffered.

        // A buffer is a temporary storage area for data that is being sent from one place to another. It is used to store data while it is being moved from one place to another. It is a temporary storage area.

        stdin() // stdin is a function that reads from the standard input
            .read_line(&mut user_text)
            .expect("Failed to read line"); // error handling

        println!("");

        let sp = Spinner::new(&Spinners::Dots12, "\t\tOpenAI is Thinking...".into()); // spinner from spinners crate that displays "Thinking..." next to a spinner

        // making the OpenAI API request
        let oai_request = OAIRequest {
            prompt: format!("{} {}", preamble, user_text), // formatting the prompt
            max_tokens: 1000, // max tokens in the response, 100 to see the entire response
        };

        // body of http request, serialized version of OpenAI API request struct
        let body = Body::from(serde_json::to_vec(&oai_request)?);

        // http request, provided by hyper crate, the POST request
        // include our body and headers that OpenAI requires
        let req = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json") // In requests, (such as POST or PUT), the client tells the server what type of data is being sent.
            .header("Authorization", &auth_header_val) // takes in a key and a value
            .body(body) // passing in the body
            .unwrap(); // if value is none, panic

        // using hyper client to pass in the request and tell it to await the response, storing the result in res
        let res = client.request(req).await?;

        // getting the body of the response, storing it in the variable body
        let body = hyper::body::aggregate(res).await?; // Aggregate the data buffers from a body asynchronously (hence await). Waiting for all the chunks of data to come back and pull a body out of that.

        // deserializing the body into the OpenAI API response struct
        let json: OAIResponse = serde_json::from_reader(body.reader())?;

        // we've got response by now, stop the spinner

        // stopping the spinner
        sp.stop();

        println!(""); // println!: same as print! but a newline is appended.

        // printing the response, choices is coming from the OpenAI API response struct, so we are accessing the choices vector and getting the first one
        // choices is a possible list of responses, we are getting the first one
        println!("{}", json.choices[0].text);
    }

    Ok(())
}
