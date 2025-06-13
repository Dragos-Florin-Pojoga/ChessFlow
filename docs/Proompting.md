
> I am master proompt enjeniir, certified in advanced keyboard smashing.
>
> Spelling is optional, genius is mandatory.

---

### AIs used[^ranking]:
[^ranking]: ranked based on my experience with them during this project. The results are heavily biased because of the mostly Rust codebase and very specific requirements

* [Gemini 2.5 Pro](https://gemini.google.com)
* [Gemini 2.5 Flash](https://gemini.google.com)
* [DeepSeek R1](https://chat.deepseek.com)
* [ChatGPT 4o](https://chatgpt.com)
* [Cursor AI](https://www.cursor.com)
    * with [Claude Sonnet 3.5](https://claude.ai)
* [Claude Sonnet 4](https://claude.ai/new)
* [DeepSeek V3](https://chat.deepseek.com)
* [ChatGPT o4-mini](https://chatgpt.com)
* [Github Copilot](https://github.com/features/copilot)
    * trough [Vscode agent mode](https://code.visualstudio.com) with [Claude Sonnet 3.5](https://claude.ai)
* [Microsoft Copilot](https://copilot.microsoft.com)

---

### The "ChessFlow" name was actually AI-generated.

It’s surprisingly difficult to come up with a name for a chess-related app that sounds good and isn’t already in use. We brainstormed ideas for quite some time before settling on this one. To generate ideas, we used ChatGPT and DeepSeek with very simple prompts.

---

### Researching Rust WASM threads

This task became a whole endeavor that took several days. I used ChatGPT, DeepSeek, and Gemini for research, as this is quite a niche topic with limited resources available online. Most of the prompts were like “Google searches,” asking the AI to dive into documentation and examine the code of many open-source crates to find something that could work for my use case. Finding a crate that was actually updated to work with newer Rust versions was a tough challenge, as multithreaded WASM is still extremely new.

---

### Learning Rust

Although the Rust Book and other learning resources are quite good, when encountering a new construct, it’s usually faster to just ask the AI to explain it rather than search through the docs to piece everything together. A good example of this is understanding the `Arc<Mutex<T>>` pattern.


---

### Spellchecking and rephrasing text across the whole project

For example, this entire README was proofread and polished by ChatGPT.

---

### Learning React, and Integration

This endeavour involved a lot of different components that I hadn't used before, alongside integrating them together such that they mesh well and communicate between each other. As such, the use of LLN agents, such as ChatGPT(4o) and Claude(4) offered a starting off point for me to learn about the libraries and packages that I was to use. The main component that I learned through this method was the React library, in which (almost) the entirety of the Client part of the website is written in. The LLMs also helped me figure out why certain bugs or inaccuracies appeared, mainly on the React part, figuring out why certain components weren't rendering/rendering twice or State variables weren't being saved/transmitted properly.
#### Comparison
Between the 2 LLMs, ChatGPT was generally more useful, as it had a bigger context window and was generally able to memorize and comprehend the parts of the code I was sending or specifications I was giving, formulating responses that sometimes also suggested better approaches. Claude's separate code completions/versioning were generally able to make the code it was outputting more coherent, and allowing it not to backtrack or miss details (at least for the final shown result) as often compared to ChatGPT.