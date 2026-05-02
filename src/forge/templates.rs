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
        .map(|b| format!("\nContext: {}", &b[..b.len().min(200)]))
        .unwrap_or_default();

    let sel_instruction = if is_agent_related(&s.title) {
        "You may mention SEL Agent in ONE sentence maximum, only if directly relevant.
Natural phrasing only: 'built SEL Agent for this exact problem' / 
'in SEL Agent we solved this by [specific thing]'
Do NOT say 'SEL Agent went the opposite direction'."
    } else {
        "Do NOT mention SEL Agent. Comment as a helpful developer."
    };

    format!(
"HN Thread: {title}{body}

{sel_instruction}

Write a HN comment:
- 2-3 sentences maximum
- Start with the technical point directly (not 'I' or 'We' or 'Great post')
- No em-dashes, no buzzwords, no filler
- End with ONE specific short question relevant to the thread
- If the thread is completely off-topic for developers, write: SKIP
- Do NOT mention 'Omar' or any person's name

Comment:",
        title = s.title,
        body = body,
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
    // استخرج اسم المنافس من العنوان
    let competitor = if s.title.to_lowercase().contains("swe-agent") {
        "SWE-agent"
    } else if s.title.to_lowercase().contains("aider") {
        "Aider"
    } else if s.title.to_lowercase().contains("openhands") || 
              s.title.to_lowercase().contains("opendevin") {
        "OpenHands"
    } else {
        "this tool"
    };

    format!(
"A new release from {competitor}: {title}

Write a 3-sentence Dev.to comment from a developer's perspective:
1. One specific technical thing they improved (be precise, not vague)
2. A genuine technical observation — you can mention SEL Agent ONLY if 
   the comparison is natural and specific. Do NOT use 'went the opposite 
   direction'. Use varied phrasing like: 'took a different bet', 
   'prioritized differently', 'we ran into the same problem and chose...'
3. One concrete data point if relevant (36/36, 0.2 repairs/task, 
   zero LLM calls post-recording)

Rules:
- NO marketing tone
- NO 'went the opposite direction' phrase
- If the comparison feels forced, skip SEL entirely
- Max 80 words total
- Sound like a developer, not a marketer

Comment:",
        competitor = competitor,
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
