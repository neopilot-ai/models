# Free Models Provider

This provider contains a comprehensive collection of 50+ top free and open-source AI models organized by category.

## Model Categories

### Large Language Models (LLMs)

#### Meta Llama Family
- **Meta-Llama-3.1-405B-Instruct** - 405B parameters, state-of-the-art performance
- **Meta-Llama-3.1-70B-Instruct** - 70B parameters, excellent balance of quality and efficiency
- **Meta-Llama-3.1-8B-Instruct** - 8B parameters, lightweight and fast

#### Mistral Family
- **Mistral-7B-Instruct-v0.3** - 7B parameters, fast and efficient
- **Mistral-Large-Instruct-2407** - 176B parameters via Mixture of Experts, high quality

#### Qwen Family
- **Qwen2.5-72B-Instruct** - 72B parameters, multilingual support
- **Qwen2.5-32B-Instruct** - 32B parameters, balanced performance
- **Qwen2.5-7B-Instruct** - 7B parameters, lightweight option
- **Qwen2.5-Coder-32B-Instruct** - Specialized for code generation
- **Qwen2.5-Coder-7B-Instruct** - Small specialized coder model
- **QwQ-32B-Preview** - 32B parameters with advanced reasoning capabilities
- **GLM-4-9B** - Multilingual model with 128K context window

#### DeepSeek Family
- **DeepSeek-V3** - Latest flagship model with advanced capabilities
- **DeepSeek-R1** - Reasoning-focused model with chain-of-thought
- **DeepSeek-Coder-33B-Instruct** - Specialized for programming tasks

#### Other Major Models
- **Phi-4** - Microsoft's latest compact model
- **Phi-3.5-Mini-Instruct** - Small efficient model with 128K context
- **Gemma-2-27B-Instruct** - 27B parameters by Google
- **Gemma-2-9B-Instruct** - 9B parameters, very efficient
- **Gemma-2-2B-Instruct** - 2B parameters, ultra-lightweight
- **OLMo-7B-Instruct** - Open-source model by Allen Institute
- **Falcon-180B-Chat** - 180B parameters, multilingual
- **Falcon-40B-Instruct** - 40B parameters, efficient
- **Nous-Hermes-3-Llama-8B** - 8B parameters, high quality
- **Mixtral-8x7B-Instruct-v0.1** - Mixture of Experts model, 8x7B
- **Dolphin-2.9-Mixtral-8x7B** - Uncensored variant of Mixtral
- **Yi-34B-Chat** - 34B parameters with 200K context
- **Baichuan2-13B-Chat** - 13B parameters by Baichuan
- **Bloom-176B** - 176B multilingual model
- **MPT-30B-Instruct** - 30B parameters by MosaicML
- **Orca-2-13B** - 13B reasoning-focused model
- **WizardLM-2-8x22B** - Mixture of Experts, 65K context
- **Zephyr-7B-Beta** - 7B parameters, optimized for chat
- **Starling-LM-7B-Beta** - 7B parameters, conversation optimized
- **Solar-10.7B-Instruct-v1.0** - 10.7B parameters
- **Intel-Neural-Chat-7B-v3.3** - 7B parameters by Intel
- **StableLM-Zephyr-3B** - 3B parameters, ultra-compact
- **Neural-Chat-7B-v3** - 7B parameters for chat
- **Openchat-3.5** - 7B parameters, high quality
- **Code Llama-34B-Instruct** - 34B specialized for code
- **Starcoder2-15B-Instruct** - 15B specialized for code
- **Grok-2-1212** - Latest Grok model by xAI
- **Jamba-1.5-Large** - Large model with 256K context
- **Goliath-120B** - 120B parameters model
- **ORCA-Mini-LLaMA-3B** - 3B parameters, ultra-lightweight

### Embedding Models (Semantic Search & Retrieval)

- **All-MiniLM-L6-v2** - Fast and efficient, 384-dim embeddings
- **multilingual-e5-large** - Multilingual embeddings
- **e5-base-v2** - Base model for embeddings
- **bge-large-en-v1.5** - Large English embeddings
- **bge-small-en-v1.5** - Small efficient embeddings
- **all-MiniLM-L12-v2** - Extended version with 384-dim output

### Vision & Multimodal Models

- **LLaVA-1.6-Mistral-7B** - Vision language model combining Llava with Mistral
- **Qwen-VL-Plus** - Qwen's vision model with 32K context

## Features

- All models have **zero cost** (input and output)
- Open-source and freely available weights for most models
- Support for various capabilities:
  - Text generation
  - Code generation and understanding
  - Multimodal (text + image)
  - Embedding generation
  - Advanced reasoning
  - Tool calling
  - Temperature control

## Provider Configuration

```toml
name = "Free Models"
env = []
npm = "@ai-sdk/openai-compatible"
doc = "https://huggingface.co"
api = "https://api-inference.huggingface.co/models"
```

## Usage

The models follow a standard TOML format with the following structure:

```toml
name = "Model-Name"
family = "model-family"
release_date = "YYYY-MM-DD"
last_updated = "YYYY-MM-DD"
attachment = false  # supports images/files
reasoning = false   # advanced reasoning capability
temperature = true  # temperature control
tool_call = true    # function calling support
open_weights = true # open-source weights

[cost]
input = 0
output = 0

[limit]
context = 128_000      # context window size
output = 8_192         # max output tokens

[modalities]
input = ["text", "image"]
output = ["text"]
```

## Model Selection Guide

- **Best for Production**: Meta-Llama-3.1-70B, Mistral-Large, Qwen2.5-72B
- **Best for Speed**: Phi-3.5-Mini, StableLM-Zephyr-3B, Gemma-2-9B
- **Best for Code**: DeepSeek-Coder, Code-Llama, Starcoder2
- **Best for Reasoning**: QwQ-32B, DeepSeek-R1, Orca-2-13B
- **Best for Vision**: LLaVA-1.6-Mistral, Qwen-VL-Plus
- **Best for Embeddings**: bge-large-en, multilingual-e5-large
- **Best Budget**: Phi-4, Gemma-2-2B, StableLM-Zephyr-3B

## Total Models: 50+

This provider includes over 50 carefully selected models across all major categories to support various use cases from small edge devices to large-scale deployments.
