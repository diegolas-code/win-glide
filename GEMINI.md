# `win-glide` Project Instructions

* Check the [Copilot Instructions](.github/copilot-instructions.md) for detailed guidelines on how to contribute to this project effectively.

* Check the [Idea Document](idea.md) for the overall concept and design philosophy of `win-glide`.

* Check the [Specification Document](spec.md) for the technical requirements and expected behavior of the application.

* Check the [TODO List](TODO.md) for the current implementation roadmap and task tracking.

## Branch Management & Merging

*   **Explicit Approval Required**: NEVER merge a feature, fix, or experimental branch into `master`, `main`, or `dev` without obtaining explicit permission from the user first.
*   **Workflow**: Always implement features in a dedicated branch, verify them, and then ask the user if they are ready to merge.

## Documentation & History Workflow

After every major commit or phase completion, you MUST:
1.  **Update `TODO.md`**: Mark completed tasks and add any newly discovered sub-tasks.
2.  **Update `pause.md`**: Summarize the current status, recent achievements, and immediate next steps.
3.  **Create a History Log**: Create a new file in `.history/history_NNN.md` (incrementing the number) detailing technical decisions, root causes found, and significant changes.
4.  **Commit Docs**: Commit these documentation changes separately with a clear message (e.g., `docs: update history and todo for [Feature/Phase]`).