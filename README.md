# PPQ Assistant

A command-line tool for interacting with AI models and executing code snippets directly from their responses. It uses the [ppq.ai](https://ppq.ai) API, which is a proxy for many cutting-edge LLMs that doesn't require a subscription, but lets you pre-pay for queries using Bitcoin, Credit Card, or altcoins.

## Configuration

Create a configuration file at `~/.ppq/config.json` with the following structure:

```json
{
    "api_token": "your-api-token-here",
    "api_url": "https://my-custom-api.xyz", // Optional, defaults to https://api.ppq.ai/chat/completions
    "default_model": "some-model" // Optional, defaults to claude-3.7-sonnet, see ppq.ai docs for available models
}
```

Only the `api_token` field is required. The other fields will use their default values if omitted. You can get your API token from [ppq.ai](https://ppq.ai) by clicking "Get an API key!" on the lower left.

## Usage

Just write your prompt as the argument to `ppq`. The prompt will be sent to PPQ and all executable code snippets contained in the response will be displayed with previews. You can then select a snippet to execute either using arrow keys or by entering the number of the snippet. See the full list of options below.

```bash
$ ppq Produce 5 hello world scripts in bash, with 1-5 lines
Select a code snippet to execute:
 0 : Bash snippet (1 lines)
     echo "Hello, World!"

[1]: Bash snippet (2 lines)
     #!/bin/bash
     echo "Hello, World!"

 2 : Bash snippet (3 lines)
     #!/bin/bash
     message="Hello, World!"
     echo $message

 3 : Bash snippet (4 lines)
     #!/bin/bash
     greeting="Hello"
     object="World"
     ...

 4 : Bash snippet (5 lines)
     #!/bin/bash
     # A simple Hello World script
     greeting="Hello"
     ...


Executing Bash snippet...

Hello, World!

Execution completed successfully.
```

## Available Options

```
Usage: ppq-assistant [OPTIONS] <prompt>...

Arguments:
  <prompt>...  

Options:
      --model <model>  [default: claude-3.7-sonnet] [possible values: deepseek-r1, gpt-4.5-preview, deepseek-chat, claude-3.7-sonnet, claude-3.5-sonnet, gpt-4o, llama-3.1-405b-instruct, llama-3-70b-instruct, gpt-4o-mini, gemini-flash-1.5, mixtral-8x7b-instruct, claude-3-5-haiku-20241022:beta, gemini-2.0-flash-exp, grok-2, qwq-32b-preview, nova-pro-v1, llama-3.1-nemotron-70b-instruct, gemini-flash-1.5, gpt-4, dolphin-mixtral-8x22b]
  -h, --help           Print help
```
