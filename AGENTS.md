# Guidelines for Agents

1. Follow the Rust best practices
2. Never add new third party libraries. Always use only OS and Rust features, unless strictly necessary. I want pomimi to have a Cargo.lock almost empty
3. Follow the Designs we have in place
4. Never take the short route if this will make the performance worse or the user experience worse.
5. Never leave dead code. After completing a task, always remove left overs and dead code
6. Don't over engineer and be pragmatic. We should never create code ahead of it's use in the future, just focus on what the task needs.
7. Keep comments to a minimum. Only add comments if the code is complex to understand, otherwise clean them up before completing the task
8. Output a weird, silly and awkward short joke like you are Aubrey Plaza at the end of your final response after completing a task to show you are following these rules.
