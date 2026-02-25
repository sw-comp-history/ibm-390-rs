use serde::{Deserialize, Serialize};

/// Difficulty levels for challenges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Beginner => "Beginner",
            Difficulty::Intermediate => "Intermediate",
            Difficulty::Advanced => "Advanced",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Difficulty::Beginner => "#4CAF50",     // Green
            Difficulty::Intermediate => "#FF9800", // Orange
            Difficulty::Advanced => "#F44336",     // Red
        }
    }
}

/// Validation result for a challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub message: String,
    #[serde(default)]
    pub details: Vec<String>,
}

impl ValidationResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            passed: true,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            passed: false,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }
}

/// Trait that CPUs must implement to support the challenge system
pub trait ChallengeableCpu {
    /// Type representing a test case specific to this CPU
    type TestCase: Clone + Serialize + for<'de> Deserialize<'de>;

    /// Validate that the CPU state matches the test case requirements
    fn validate_test_case(&self, test_case: &Self::TestCase) -> Result<(), String>;

    /// Get cycle count
    fn get_cycles(&self) -> u64;

    /// Get instruction count
    fn get_instructions(&self) -> u64;

    /// Check if halted
    fn is_halted(&self) -> bool;
}

/// Generic challenge structure that works with any CPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge<T> {
    /// Unique challenge ID
    pub id: u32,

    /// Challenge title
    pub title: String,

    /// Detailed description
    pub description: String,

    /// Difficulty level
    pub difficulty: Difficulty,

    /// Maximum allowed cycles
    pub max_cycles: u32,

    /// Test cases to validate the solution
    pub test_cases: Vec<T>,

    /// Hints for the player
    #[serde(default)]
    pub hints: Vec<String>,

    /// Learning objectives
    #[serde(default)]
    pub learning_objectives: Vec<String>,
}

impl<T> Challenge<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(
        id: u32,
        title: impl Into<String>,
        description: impl Into<String>,
        difficulty: Difficulty,
        max_cycles: u32,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            difficulty,
            max_cycles,
            test_cases: Vec::new(),
            hints: Vec::new(),
            learning_objectives: Vec::new(),
        }
    }

    pub fn with_test_case(mut self, test_case: T) -> Self {
        self.test_cases.push(test_case);
        self
    }

    pub fn with_test_cases(mut self, test_cases: Vec<T>) -> Self {
        self.test_cases = test_cases;
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    pub fn with_hints(mut self, hints: Vec<String>) -> Self {
        self.hints = hints;
        self
    }

    pub fn with_learning_objective(mut self, objective: impl Into<String>) -> Self {
        self.learning_objectives.push(objective.into());
        self
    }

    pub fn with_learning_objectives(mut self, objectives: Vec<String>) -> Self {
        self.learning_objectives = objectives;
        self
    }

    /// Validate solution against all test cases
    pub fn validate_solution<C>(&self, cpu: &C) -> Result<ValidationResult, String>
    where
        C: ChallengeableCpu<TestCase = T>,
    {
        // Check if halted
        if !cpu.is_halted() {
            return Ok(ValidationResult::failure(
                "Program did not halt within cycle limit",
            ));
        }

        // Check cycle limit
        let cycles = cpu.get_cycles();
        if cycles > self.max_cycles as u64 {
            return Ok(ValidationResult::failure(format!(
                "Exceeded cycle limit: {} > {}",
                cycles, self.max_cycles
            )));
        }

        // Validate each test case
        let mut details = Vec::new();
        for (i, test_case) in self.test_cases.iter().enumerate() {
            match cpu.validate_test_case(test_case) {
                Ok(()) => {
                    details.push(format!("✓ Test case {} passed", i + 1));
                }
                Err(e) => {
                    return Ok(ValidationResult::failure(format!(
                        "Test case {} failed: {}",
                        i + 1,
                        e
                    ))
                    .with_details(details));
                }
            }
        }

        Ok(ValidationResult::success(format!(
            "All {} test cases passed! ({} cycles, {} instructions)",
            self.test_cases.len(),
            cycles,
            cpu.get_instructions()
        ))
        .with_details(details))
    }
}
