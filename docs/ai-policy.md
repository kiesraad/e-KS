# AI Usage Policy for Software Development

## 1. Purpose

This policy defines how Artificial Intelligence (AI) tools may be used in this project.  
AI is a powerful productivity aid, but it must be used in a way that preserves code quality, maintainability, and deep understanding of the system.

Our primary principle:

> AI may assist development, but it must never replace developer understanding.

---

## 2. Guiding Principles

- Developers remain fully responsible for all code they commit.
- Every committed line of code must be understood by the developer.
- AI is a tool for acceleration — not a substitute for thinking.
- Maintainability and shared understanding of the system take priority over short-term speed.

---

## 3. Prohibited Practice: “Vibe-Coding”

### Definition

“Vibe-coding” is defined as:

> Generating functionalities or large blocks of code using AI that you would not have written yourself in the same manner, and committing them without fully understanding and being able to reproduce the logic.

### Why It Is Not Allowed

Vibe-coding leads to:

- Shallow understanding of the system
- Weak or incomplete mental models
- Bugs that are not recognized during implementation
- Harder debugging
- Increasing maintenance complexity over time
- Reduced long-term code ownership

Software quality depends on developers deeply understanding what they build.  
If you cannot confidently explain, modify, or rewrite the generated code yourself, it must not be committed.

**Therefore: vibe-coding is not allowed.**

---

## 4. Allowed AI Use Cases

AI may be used in the following limited and explicit ways:

### 4.1 Search

Using AI as an advanced search tool is allowed, including:

- Searching through the codebase
- Finding relevant modules or patterns
- Discovering dependencies
- Looking up documentation
- Exploring unfamiliar parts of the system

The developer must still verify results and interpret them critically.

---

### 4.2 Autocomplete

AI may be used to:

- Complete trivial lines of code
- Suggest small syntactic continuations
- Fill obvious implementations that follow directly from context

This is comparable to advanced autocomplete.  
The developer must already understand the logic being implemented.

---

### 4.3 Automation

AI may be used to automate repetitive or mechanical tasks, such as:

- Boilerplate test generation
- Simple refactors
- Renaming across files
- Formatting adjustments
- Converting between equivalent structures
- Rewriting repetitive patterns

These tasks must be:

- Easy to review
- Fully understood by the developer

---

## 5. Developer Responsibility

Before committing AI-assisted code, the developer must ensure:

- They understand the full logic of the code.
- They could reasonably recreate it without AI.
- They can debug it confidently.
- It aligns with existing architectural patterns.
- It meets project quality standards.

If any of these are not true, the changes must be reverted or rewritten manually.

---

## 6. Long-Term Maintainability Over Short-Term Speed

AI can improve velocity, but unmanaged use degrades code quality over time.  
We prioritize:

- Clear architecture
- Strong mental models
- Predictable behavior
- Long-term maintainability

AI is a multiplier of developer capability — not a replacement for it.

---

## 7. Summary

Allowed:
- Search
- Trivial autocomplete
- Mechanical automation

Not allowed:
- Generating and committing substantial functionality you do not fully understand ("vibe-coding")

When in doubt:  
If you would not have written it yourself, do not commit it.
