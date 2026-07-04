pub(super) const VALID_GRID_GAME: &str = r#"
id: game_test_grid
title: Test Grid
description: A test grid game
games:
  - mode: grid_quiz
    title: Round 1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: this_question
      steal_policy: round_limited
      judge: auto
      question_timer_secs: 30
      answer_timer_secs: 15
    board:
      points: [100, 200, 300, 500]
      categories:
        - name: Capitals
          filter:
            tags_any: [subject:geo]
        - name: Flags
          question_ids: { 100: q_alpha_one, 200: q_alpha_two }
"#;

pub(super) const VALID_LINEAR_GAME: &str = r#"
id: game_test_linear
title: Test Linear
description: A test linear game
games:
  - mode: linear
    title: Round 1
    rules:
      buzz_policy: broadcast
      scoring_mode: all_grade
      lockout_policy: none
      steal_policy: none
      judge: auto
      question_timer_secs: 20
      answer_timer_secs: 10
    questions:
      source: questions
      question_ids: [q_alpha_one]
"#;

pub(super) const VALID_QUESTION_TWO: &str = r#"
- id: q_alpha_two
  kind: text
  tags: [subject:history, difficulty:general]
  content:
    default_lang: en
    prompt: { text: "What is two plus two?" }
    answer: Four
    variants:
      open:
        accepted: ["Four", "4"]
"#;
