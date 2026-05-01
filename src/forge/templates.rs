use crate::models::{Platform, Signal, SignalSource};

pub fn select_platform(signal: &Signal) -> Platform {
    match &signal.source {
        SignalSource::RedditLocalllama
        | SignalSource::RedditProgramming
        | SignalSource::RedditMachinelearning
        | SignalSource::RedditRust        => Platform::Reddit,
        SignalSource::HackerNews          => Platform::HackerNews,
        SignalSource::GithubCompetitor(_) => Platform::DevTo,
    }
}

pub fn build_prompt(signal: &Signal, platform: &Platform) -> String {
    match platform {
        Platform::Reddit     => reddit_prompt(signal),
        Platform::HackerNews => hn_prompt(signal),
        Platform::DevTo      => devto_prompt(signal),
        Platform::GitHub     => readme_prompt(),
        Platform::Twitter    => twitter_prompt(signal),
    }
}

fn is_agent_related(title: &str) -> bool {
    let t = title.to_lowercase();
    // مباشرة عن coding agents
    let direct = t.contains("coding agent")
        || t.contains("code agent")
        || t.contains("autonomous cod")
        || t.contains("swe-agent")
        || t.contains("agentic cod")
        || t.contains("devin")
        || t.contains("code repair")
        || t.contains("ai programmer");
    // أدوات تتعلق بالـ agents لكن ليست عنها مباشرة
    // credential broker, runbooks, etc. = false
    direct
}

fn hn_prompt(s: &Signal) -> String {
    let body = s.body.as_deref()
        .map(|b| format!("\nContext: {}", &b[..b.len().min(150)]))
        .unwrap_or_default();

    let sel_instruction = if is_agent_related(&s.title) {
        "You may mention SEL Agent in one sentence if it adds value.
Use natural phrasing: built SEL Agent for this / SEL Agent solves this by /
in SEL Agent we found that..."
    } else {
        "Do NOT mention SEL Agent. Comment purely as a developer."
    };

    format!(
"Thread: {title}{body}

{sel_instruction}

Write a HN comment:
- 2-3 sentences maximum
- Start with the actual point, not with I or We
- No em-dashes, no filler phrases
- End with one short specific question
- If the thread is not about software at all, write: SKIP

Comment:",
        title = s.title,
        body  = body,
        sel_instruction = sel_instruction,
    )
}

fn reddit_prompt(s: &Signal) -> String {
    let body = s.body.as_deref()
        .map(|b| format!("\n{}", &b[..b.len().min(200)]))
        .unwrap_or_default();

    let sel_instruction = if is_agent_related(&s.title) {
        "Mention SEL Agent naturally in one sentence if helpful."
    } else {
        "Do NOT mention SEL Agent."
    };

    format!(
"Reddit thread: {title}{body}

{sel_instruction}
2-3 sentences. Helpful and direct. No marketing tone.
If off-topic write: SKIP

Comment:",
        title = s.title,
        body  = body,
        sel_instruction = sel_instruction,
    )
}

fn devto_prompt(s: &Signal) -> String {
    format!(
"Competitor release: {title}

Write 3 sentences:
1. What they did well (specific, not vague)
2. One technical difference: SEL Agent trades X for Y by...
   or: SEL Agent went the opposite direction...
   or: we prioritized X over Y in SEL Agent...
3. One honest number: 36/36 bench or 0.2 repairs or 100% mutation score

No hype. 3 sentences only.",
        title = s.title,
    )
}

fn readme_prompt() -> String {
    "GitHub README Benchmarks section — GFM Markdown only:

| Language   | Cases | Passed | Score |
|------------|-------|--------|-------|
| Python     | 9     | 9      | 100%  |
| Go         | 9     | 9      | 100%  |
| TypeScript | 9     | 9      | 100%  |
| Rust       | 9     | 9      | 100%  |
| **Total**  | **36**| **36** |**100%**|

Mutation score: 100% | Avg repairs: 0.2 | Replay: zero LLM calls post-recording.".to_string()
}

fn twitter_prompt(s: &Signal) -> String {
    format!(
"Tweet about: {title}

Max 240 chars. One number (36/36 or 0.2 or 100%). Max 1 hashtag.
Strong first 4 words. End with [link]. No em-dashes.

Tweet:",
        title = s.title,
    )
}
