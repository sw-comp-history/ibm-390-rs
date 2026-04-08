//! Yew application for IBM ESA/390 Assembly Game

use components::{
    Header, LegendItem, MemoryViewer, Modal, ProgramArea, Register, RegisterPanel, Sidebar,
    SidebarButton,
};
use yew::prelude::*;

use crate::wasm::WasmCpu;

#[function_component(App)]
pub fn app() -> Html {
    // State management
    let cpu = use_state(|| WasmCpu::new());
    let program_code = use_state(|| String::from(EXAMPLE_PROGRAM));
    let assembly_output = use_state(|| None::<Html>);
    let assembly_lines = use_state(|| Vec::<String>::new());
    let last_registers = use_state(|| vec![0u32; 16]);
    let challenge_mode = use_state(|| false);
    let current_challenge_id = use_state(|| None::<u32>);
    let challenge_result = use_state(|| None::<Result<String, String>>);

    // Modal states
    let tutorial_open = use_state(|| false);
    let examples_open = use_state(|| false);
    let challenges_open = use_state(|| false);
    let isa_ref_open = use_state(|| false);
    let help_open = use_state(|| false);

    // Sidebar buttons
    // Callbacks for modals
    let close_tutorial = {
        let tutorial_open = tutorial_open.clone();
        Callback::from(move |_| tutorial_open.set(false))
    };
    let close_examples = {
        let examples_open = examples_open.clone();
        Callback::from(move |_| examples_open.set(false))
    };
    let close_challenges = {
        let challenges_open = challenges_open.clone();
        Callback::from(move |_| challenges_open.set(false))
    };
    let close_isa_ref = {
        let isa_ref_open = isa_ref_open.clone();
        Callback::from(move |_| isa_ref_open.set(false))
    };
    let close_help = {
        let help_open = help_open.clone();
        Callback::from(move |_| help_open.set(false))
    };

    // Example programs
    let examples = vec![
        (
            "Example 1: Load and Add",
            "; Load value from 0x100 into R1\nL R1,0x100\n\n; Load value from 0x104 into R2\nL R2,0x104\n\n; Add R1 + R2 -> R3\nLR R3,R1\nAR R3,R2\n\n; Halt\nHIO\n\n; Data section\nDATA 0x100 42\nDATA 0x104 58",
        ),
        (
            "Example 2: Subtract with Memory",
            "; Load value from 0x100 into R1\nL R1,0x100\n\n; Subtract value at 0x104 from R1\nS R1,0x104\n\n; Store result to 0x108\nST R1,0x108\n\n; Halt\nHIO\n\n; Data section\nDATA 0x100 100\nDATA 0x104 27",
        ),
        (
            "Example 3: Register Operations",
            "; Load from memory into R1\nL R1,0x100\n\n; Copy R1 to R2\nLR R2,R1\n\n; Add R1 to R2\nAR R2,R1\n\n; Store result\nST R2,0x108\n\n; Halt\nHIO\n\n; Data section\nDATA 0x100 25",
        ),
    ];

    // Sidebar buttons with inline callbacks
    let sidebar_buttons = vec![
        SidebarButton {
            emoji: "📚".to_string(),
            label: "Tutorial".to_string(),
            onclick: {
                let tutorial_open = tutorial_open.clone();
                Callback::from(move |_| tutorial_open.set(true))
            },
            title: Some("Learn IBM ESA/390 basics".to_string()),
        },
        SidebarButton {
            emoji: "📝".to_string(),
            label: "Examples".to_string(),
            onclick: {
                let examples_open = examples_open.clone();
                Callback::from(move |_| examples_open.set(true))
            },
            title: Some("Load example programs".to_string()),
        },
        SidebarButton {
            emoji: "🎯".to_string(),
            label: "Challenges".to_string(),
            onclick: {
                let challenges_open = challenges_open.clone();
                Callback::from(move |_| challenges_open.set(true))
            },
            title: Some("Test your skills".to_string()),
        },
        SidebarButton {
            emoji: "📖".to_string(),
            label: "ISA Ref".to_string(),
            onclick: {
                let isa_ref_open = isa_ref_open.clone();
                Callback::from(move |_| isa_ref_open.set(true))
            },
            title: Some("Instruction reference".to_string()),
        },
        SidebarButton {
            emoji: "❓".to_string(),
            label: "Help".to_string(),
            onclick: {
                let help_open = help_open.clone();
                Callback::from(move |_| help_open.set(true))
            },
            title: Some("Usage help".to_string()),
        },
    ];

    // CPU operation callbacks
    let on_assemble = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let assembly_lines = assembly_lines.clone();
        let program_code = program_code.clone();

        Callback::from(move |code: String| {
            program_code.set(code.clone());

            let mut new_cpu = (*cpu).clone();

            // Assemble the complete source code (assembler handles DATA directives)
            match new_cpu.assemble(&code) {
                Ok(output_js) => {
                    match serde_wasm_bindgen::from_value::<crate::assembler::AssemblyOutput>(
                        output_js,
                    ) {
                        Ok(output) => {
                            cpu.set(new_cpu);

                            // Store disassembly lines for highlighting
                            // Add PROGRAM_START_ADDRESS to convert relative to absolute addresses
                            let program_start = (*cpu).get_program_start_address();
                            let disasm_lines: Vec<String> = output
                                .lines
                                .iter()
                                .map(|line| {
                                    format!(
                                        "0x{:08X}: {}",
                                        program_start + line.address,
                                        line.instruction.as_ref().unwrap_or(&line.source)
                                    )
                                })
                                .collect();
                            assembly_lines.set(disasm_lines);

                            assembly_output.set(Some(html! {
                                <div class="success-text">
                                    {"✓ Program assembled successfully"}
                                </div>
                            }));
                        }
                        Err(e) => {
                            assembly_lines.set(Vec::new());
                            assembly_output.set(Some(html! {
                                <div class="error-text">
                                    {format!("Error: {}", e)}
                                </div>
                            }));
                        }
                    }
                }
                Err(e) => {
                    assembly_lines.set(Vec::new());
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_step = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let last_registers = last_registers.clone();

        Callback::from(move |()| {
            // Save current state for change tracking
            if let Ok(state_js) = (*cpu).get_state() {
                if let Ok(state) =
                    serde_wasm_bindgen::from_value::<crate::wasm::RegisterState>(state_js)
                {
                    last_registers.set(vec![
                        state.r0, state.r1, state.r2, state.r3, state.r4, state.r5, state.r6,
                        state.r7, state.r8, state.r9, state.r10, state.r11, state.r12, state.r13,
                        state.r14, state.r15,
                    ]);
                }
            }

            let mut new_cpu = (*cpu).clone();
            match new_cpu.step() {
                Ok(_) => {
                    cpu.set(new_cpu);
                    // Don't set assembly_output - let the highlighting show in the main display
                }
                Err(e) => {
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_run = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();

        Callback::from(move |()| {
            let mut new_cpu = (*cpu).clone();
            match new_cpu.run(10000) {
                Ok(_) => {
                    cpu.set(new_cpu);
                    assembly_output.set(Some(html! {
                        <div class="success-text">
                            {"✓ Program completed"}
                        </div>
                    }));
                }
                Err(e) => {
                    assembly_output.set(Some(html! {
                        <div class="error-text">
                            {format!("Error: {:?}", e)}
                        </div>
                    }));
                }
            }
        })
    };

    let on_reset = {
        let cpu = cpu.clone();
        let assembly_output = assembly_output.clone();
        let assembly_lines = assembly_lines.clone();

        Callback::from(move |()| {
            // Full reset - create new CPU with cleared memory
            cpu.set(WasmCpu::new());
            assembly_lines.set(Vec::new());
            assembly_output.set(None);
        })
    };

    // Get CPU state
    let cpu_state = match (*cpu).get_state() {
        Ok(js_value) => {
            match serde_wasm_bindgen::from_value::<crate::wasm::RegisterState>(js_value) {
                Ok(state) => Some(state),
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    // Register panel data
    let registers = if let Some(ref state) = cpu_state {
        let last_regs = &*last_registers;
        let curr_regs = vec![
            state.r0, state.r1, state.r2, state.r3, state.r4, state.r5, state.r6, state.r7,
            state.r8, state.r9, state.r10, state.r11, state.r12, state.r13, state.r14, state.r15,
        ];

        (0..16)
            .map(|i| {
                let value = curr_regs[i];
                let changed = value != last_regs.get(i).copied().unwrap_or(0);
                Register {
                    name: format!("R{}", i),
                    value: format!("0x{:08X} ({})", value, value as i32),
                    changed,
                }
            })
            .collect()
    } else {
        vec![]
    };

    let legend_items = vec![
        LegendItem {
            label: "R12".to_string(),
            value: "Base Register".to_string(),
            changed: false,
        },
        LegendItem {
            label: "R13".to_string(),
            value: "Save Area Ptr".to_string(),
            changed: false,
        },
        LegendItem {
            label: "R14".to_string(),
            value: "Return Address".to_string(),
            changed: false,
        },
        LegendItem {
            label: "R15".to_string(),
            value: "Return Code".to_string(),
            changed: false,
        },
    ];

    // Memory data - show program start area
    let program_start = (*cpu).get_program_start_address();
    let memory = match (*cpu).get_memory(program_start, 128) {
        Ok(js_value) => match serde_wasm_bindgen::from_value::<Vec<u8>>(js_value) {
            Ok(mem) => mem,
            Err(_) => vec![0u8; 128],
        },
        Err(_) => vec![0u8; 128],
    };

    let pc = cpu_state.as_ref().map(|s| s.pc as u16).unwrap_or(0);

    html! {
        <div class="container">
            <Header title="IBM ESA/390 Assembly Game" />

            <Sidebar buttons={sidebar_buttons} />

            <div class="main-content">
                <ProgramArea
                    on_assemble={on_assemble}
                    on_step={on_step}
                    on_run={on_run}
                    on_reset={on_reset}
                    assembly_output={
                        if !assembly_lines.is_empty() {
                            // Show highlighted assembly lines
                            let pc = cpu_state.as_ref().map(|s| s.pc).unwrap_or(0);
                            Some(html! {
                                <div>
                                    {for assembly_lines.iter().map(|line| {
                                        // Parse address from "0xADDRESS: INSTRUCTION" format
                                        let addr_str = line.split(':').next().unwrap_or("");
                                        let is_current = if let Ok(addr) = u32::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                                            addr == pc
                                        } else {
                                            false
                                        };

                                        let class = if is_current {
                                            "assembly-line current"
                                        } else {
                                            "assembly-line"
                                        };

                                        html! {
                                            <div class={class}>{line}</div>
                                        }
                                    })}
                                </div>
                            })
                        } else {
                            // Show success/error messages
                            (*assembly_output).clone()
                        }
                    }
                    initial_code={Some((*program_code).clone())}
                    step_enabled={!(*cpu).is_halted()}
                    run_enabled={!(*cpu).is_halted()}
                />

                <div class="right-panels">
                    <div class="registers-panel">
                        <RegisterPanel
                            registers={registers}
                            legend_items={legend_items}
                        />

                        // Condition Codes
                        <div class="flags">
                            {
                                html! {
                                    <>
                                        <div class="flag">
                                            <div class={if cpu_state.as_ref().map(|s| s.cc == 0).unwrap_or(false) { "flag-indicator set" } else { "flag-indicator" }}></div>
                                            <span>{"Zero"}</span>
                                        </div>
                                        <div class="flag">
                                            <div class={if cpu_state.as_ref().map(|s| s.cc == 1).unwrap_or(false) { "flag-indicator set" } else { "flag-indicator" }}></div>
                                            <span>{"Low"}</span>
                                        </div>
                                        <div class="flag">
                                            <div class={if cpu_state.as_ref().map(|s| s.cc == 2).unwrap_or(false) { "flag-indicator set" } else { "flag-indicator" }}></div>
                                            <span>{"High"}</span>
                                        </div>
                                        <div class="flag">
                                            <div class={if cpu_state.as_ref().map(|s| s.cc == 3).unwrap_or(false) { "flag-indicator set" } else { "flag-indicator" }}></div>
                                            <span>{"Overflow"}</span>
                                        </div>
                                        <div class="flag">
                                            <div class={if cpu_state.as_ref().map(|s| s.wait).unwrap_or(false) { "flag-indicator set" } else { "flag-indicator" }}></div>
                                            <span>{"Wait"}</span>
                                        </div>
                                        <div class="flag">
                                            <div class={if cpu_state.as_ref().map(|s| s.addressing_mode_31bit).unwrap_or(false) { "flag-indicator set" } else { "flag-indicator" }}></div>
                                            <span>{"31-bit Mode"}</span>
                                        </div>
                                    </>
                                }
                            }
                        </div>

                        // CPU Status
                        <div class="cpu-status">
                            <div class="status-item">
                                <span class="status-label">{"Cycles:"}</span>
                                <span class="status-value">{cpu_state.as_ref().map(|s| s.cycles).unwrap_or(0)}</span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">{"Instructions:"}</span>
                                <span class="status-value">{cpu_state.as_ref().map(|s| s.instructions).unwrap_or(0)}</span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">{"Status:"}</span>
                                <span class="status-value">
                                    {if cpu_state.as_ref().map(|s| s.halted).unwrap_or(false) { "HALTED" } else { "RUNNING" }}
                                </span>
                            </div>
                        </div>
                    </div>

                    <MemoryViewer
                        memory={memory}
                        pc={pc}
                        title={Some(format!("Memory @ 0x{:08X} (First 128 Bytes)", program_start))}
                        bytes_per_row={16}
                        bytes_to_show={128}
                    />
                </div>
            </div>

            // Challenge Mode Banner
            if *challenge_mode {
                if let Some(challenge_id) = *current_challenge_id {
                    <div class="challenge-banner">
                        <span class="challenge-indicator">{"⚡"}</span>
                        <span class="challenge-text">
                            {format!("Challenge Mode - Challenge {}", challenge_id)}
                        </span>
                        <button
                            class="check-button"
                            onclick={
                                let cpu = cpu.clone();
                                let challenge_result = challenge_result.clone();
                                Callback::from(move |_| {
                                    match (*cpu).check_challenge(challenge_id) {
                                        Ok(result_js) => {
                                            match serde_wasm_bindgen::from_value::<asm_game_shared::ValidationResult>(result_js) {
                                                Ok(validation) => {
                                                    if validation.passed {
                                                        let message = format!("✅ Challenge {} PASSED!\n\n{}", challenge_id, validation.message);
                                                        challenge_result.set(Some(Ok(message)));
                                                    } else {
                                                        let message = format!("❌ Challenge {} did not pass.\n\n{}", challenge_id, validation.message);
                                                        challenge_result.set(Some(Err(message)));
                                                    }
                                                }
                                                Err(e) => {
                                                    challenge_result.set(Some(Err(format!("Failed to parse validation result: {}", e))));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            challenge_result.set(Some(Err(format!("Validation error: {:?}", e))));
                                        }
                                    }
                                })
                            }
                        >
                            {"Check Solution"}
                        </button>
                        <button
                            class="exit-button"
                            onclick={
                                let challenge_mode = challenge_mode.clone();
                                let current_challenge_id = current_challenge_id.clone();
                                let challenge_result = challenge_result.clone();
                                Callback::from(move |_| {
                                    challenge_mode.set(false);
                                    current_challenge_id.set(None);
                                    challenge_result.set(None);
                                })
                            }
                        >
                            {"Exit"}
                        </button>
                    </div>
                }
            }

            // Success/Error Banners
            {
                if let Some(result) = &*challenge_result {
                    match result {
                        Ok(message) => html! {
                            <div class="success-banner">
                                <span class="banner-content">{message}</span>
                                <button
                                    class="dismiss-button"
                                    onclick={
                                        let challenge_result = challenge_result.clone();
                                        Callback::from(move |_| challenge_result.set(None))
                                    }
                                >
                                    {"×"}
                                </button>
                            </div>
                        },
                        Err(message) => html! {
                            <div class="error-banner">
                                <span class="banner-content">{message}</span>
                                <button
                                    class="dismiss-button"
                                    onclick={
                                        let challenge_result = challenge_result.clone();
                                        Callback::from(move |_| challenge_result.set(None))
                                    }
                                >
                                    {"×"}
                                </button>
                            </div>
                        }
                    }
                } else {
                    html! {}
                }
            }

            // Modals
            <Modal id="tutorial" title="Tutorial" active={*tutorial_open} on_close={close_tutorial}>
                {html! { <div>{TUTORIAL_CONTENT}</div> }}
            </Modal>

            <Modal id="examples" title="Examples" active={*examples_open} on_close={close_examples}>
                <div class="examples-list">
                    {for examples.iter().enumerate().map(|(idx, (title, code))| {
                        let program_code = program_code.clone();
                        let examples_open = examples_open.clone();
                        let cpu = cpu.clone();
                        let assembly_output = assembly_output.clone();
                        let code = code.to_string();

                        let load_example = Callback::from(move |_: MouseEvent| {
                            // Reset CPU
                            cpu.set(WasmCpu::new());
                            assembly_output.set(None);

                            // Load new code
                            program_code.set(code.clone());
                            examples_open.set(false);
                        });

                        html! {
                            <div class="example-item" key={idx} onclick={load_example}>
                                <h4>{title}</h4>
                                <p>{"Click to load this example"}</p>
                            </div>
                        }
                    })}
                </div>
            </Modal>

            <Modal id="challenges" title="Challenges" active={*challenges_open} on_close={close_challenges}>
                {render_challenges_list(challenge_mode.clone(), current_challenge_id.clone(), program_code.clone(), challenges_open.clone(), cpu.clone())}
            </Modal>

            <Modal id="isaRef" title="ISA Reference" active={*isa_ref_open} on_close={close_isa_ref}>
                {html! { <div>{ISA_REF_CONTENT}</div> }}
            </Modal>

            <Modal id="help" title="Help" active={*help_open} on_close={close_help}>
                {html! { <div>{HELP_CONTENT}</div> }}
            </Modal>

            // GitHub Corner
            <a href="https://github.com/sw-comp-history/ibm-390-rs" class="github-corner" aria-label="View source on GitHub" target="_blank">
                <svg width="80" height="80" viewBox="0 0 250 250" style="fill:#00d9ff; color:#1a1a2e; position: absolute; top: 0; border: 0; right: 0;" aria-hidden="true">
                    <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z"></path>
                    <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 C122.9,97.6 130.6,101.9 134.4,103.2" fill="currentColor" style="transform-origin: 130px 106px;" class="octo-arm"></path>
                    <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z" fill="currentColor" class="octo-body"></path>
                </svg>
            </a>

            // Footer
            <footer class="app-footer">
                <div class="footer-left">
                    <span>{"MIT License"}</span>
                    <span>{"© 2026 Michael A Wright"}</span>
                    <a href="https://software-wrighter-lab.github.io/" target="_blank">{"Blog"}</a>
                    <a href="https://discord.com/invite/Ctzk5uHggZ" target="_blank">{"Discord"}</a>
                    <a href="https://www.youtube.com/@SoftwareWrighter" target="_blank">{"YouTube"}</a>
                </div>
                <div class="footer-right">
                    <span>{format!("{} | {} | {}", env!("VERGEN_BUILD_HOST"), env!("VERGEN_GIT_SHA_SHORT"), env!("VERGEN_BUILD_TIMESTAMP"))}</span>
                </div>
            </footer>
        </div>
    }
}

// Helper function to render challenges list
fn render_challenges_list(
    challenge_mode: UseStateHandle<bool>,
    current_challenge_id: UseStateHandle<Option<u32>>,
    program_code: UseStateHandle<String>,
    challenges_open: UseStateHandle<bool>,
    cpu: UseStateHandle<WasmCpu>,
) -> Html {
    // Setup test data for challenges
    let setup_challenge_1 = {
        let cpu = cpu.clone();
        Callback::from(move |_| {
            let mut new_cpu = (*cpu).clone();
            let _ = new_cpu.set_gpr(1, 0);
            // Memory at 0x100 will be set to 42 in the challenge test
            cpu.set(new_cpu);
        })
    };

    let setup_challenge_2 = {
        let cpu = cpu.clone();
        Callback::from(move |_| {
            let new_cpu = (*cpu).clone();
            // Memory at 0x100 will be 15, at 0x104 will be 27
            cpu.set(new_cpu);
        })
    };

    let setup_challenge_3 = {
        let cpu = cpu.clone();
        Callback::from(move |_| {
            let new_cpu = (*cpu).clone();
            // Memory at 0x100 will be 6, at 0x104 will be 7
            cpu.set(new_cpu);
        })
    };

    html! {
        <div class="challenges-list">
            <h3>{"Available Challenges"}</h3>
            <div class="challenge-item">
                <button
                    class="load-challenge-btn"
                    onclick={
                        let challenge_mode = challenge_mode.clone();
                        let current_challenge_id = current_challenge_id.clone();
                        let program_code = program_code.clone();
                        let challenges_open = challenges_open.clone();
                        Callback::from(move |_| {
                            setup_challenge_1.emit(());
                            challenge_mode.set(true);
                            current_challenge_id.set(Some(1));
                            program_code.set(CHALLENGE_1_TEMPLATE.to_string());
                            challenges_open.set(false);
                        })
                    }
                >
                    {"Load Challenge 1"}
                </button>
                <p><strong>{"Challenge 1: Load a Value"}</strong></p>
                <p>{"Load the value at memory address 0x100 into register R1, then halt."}</p>
            </div>
            <div class="challenge-item">
                <button
                    class="load-challenge-btn"
                    onclick={
                        let challenge_mode = challenge_mode.clone();
                        let current_challenge_id = current_challenge_id.clone();
                        let program_code = program_code.clone();
                        let challenges_open = challenges_open.clone();
                        Callback::from(move |_| {
                            setup_challenge_2.emit(());
                            challenge_mode.set(true);
                            current_challenge_id.set(Some(2));
                            program_code.set(CHALLENGE_2_TEMPLATE.to_string());
                            challenges_open.set(false);
                        })
                    }
                >
                    {"Load Challenge 2"}
                </button>
                <p><strong>{"Challenge 2: Add Two Numbers"}</strong></p>
                <p>{"Load values from addresses 0x100 and 0x104, add them, store result at 0x108."}</p>
            </div>
            <div class="challenge-item">
                <button
                    class="load-challenge-btn"
                    onclick={
                        let challenge_mode = challenge_mode.clone();
                        let current_challenge_id = current_challenge_id.clone();
                        let program_code = program_code.clone();
                        let challenges_open = challenges_open.clone();
                        Callback::from(move |_| {
                            setup_challenge_3.emit(());
                            challenge_mode.set(true);
                            current_challenge_id.set(Some(3));
                            program_code.set(CHALLENGE_3_TEMPLATE.to_string());
                            challenges_open.set(false);
                        })
                    }
                >
                    {"Load Challenge 3"}
                </button>
                <p><strong>{"Challenge 3: Multiply Two Numbers"}</strong></p>
                <p>{"Load values from addresses 0x100 and 0x104, multiply them, store result at 0x108."}</p>
            </div>
        </div>
    }
}

// Constants for content
const EXAMPLE_PROGRAM: &str = "; Example: Load and Add
; IBM ESA/390 Assembly Language

; Load value from memory into R1
L R1, 0x100

; Add value from memory to R1
A R1, 0x104

; Store result back to memory
ST R1, 0x108

; Halt execution
HIO

; Data section (initialize memory)
DATA 0x100 15
DATA 0x104 27";

const CHALLENGE_1_TEMPLATE: &str = "; Challenge 1: Load a Value
; Load the value at memory address 0x100 into register R1, then halt

; Your code here


; Data section
DATA 0x100 42
";

const CHALLENGE_2_TEMPLATE: &str = "; Challenge 2: Add Two Numbers
; Load values from addresses 0x100 and 0x104, add them, store result at 0x108

; Your code here


; Data section
DATA 0x100 15
DATA 0x104 27
";

const CHALLENGE_3_TEMPLATE: &str = "; Challenge 3: Multiply Two Numbers
; Load values from addresses 0x100 and 0x104, multiply them, store result at 0x108

; Your code here


; Data section
DATA 0x100 6
DATA 0x104 7
";

const TUTORIAL_CONTENT: &str = r#"
<h3>Welcome to the IBM ESA/390 Assembly Game!</h3>
<p>This game teaches you assembly programming using the IBM Enterprise Systems Architecture/390.</p>

<h4>CPU Features:</h4>
<ul>
    <li><strong>16 General Purpose Registers</strong>: R0 through R15</li>
    <li><strong>4 Condition Codes</strong>: Zero, Low, High, Overflow</li>
    <li><strong>31-bit Addressing</strong>: Up to 2GB memory space</li>
    <li><strong>Rich Instruction Set</strong>: Load, Store, Arithmetic, Logical, Branch</li>
</ul>

<h4>Special Registers:</h4>
<ul>
    <li><code>R12</code> - Base Register (points to program start)</li>
    <li><code>R13</code> - Save Area Pointer</li>
    <li><code>R14</code> - Return Address</li>
    <li><code>R15</code> - Return Code</li>
</ul>

<h4>Basic Instructions:</h4>
<ul>
    <li><code>L R1, address</code> - Load word from memory into R1</li>
    <li><code>ST R1, address</code> - Store R1 to memory</li>
    <li><code>A R1, address</code> - Add memory value to R1</li>
    <li><code>S R1, address</code> - Subtract memory value from R1</li>
    <li><code>M R1, address</code> - Multiply R1 by memory value</li>
    <li><code>HIO</code> - Halt I/O (stop execution)</li>
</ul>

<h4>Example:</h4>
<pre>// Load value from 0x100 into R1
L R1, 0x100

// Add value from 0x104 to R1
A R1, 0x104

// Store result to 0x108
ST R1, 0x108

// Halt
HIO</pre>
"#;

const ISA_REF_CONTENT: &str = r#"
<h3>IBM ESA/390 Instruction Set Reference</h3>

<h4>Load/Store Instructions</h4>
<p><strong>L R1, D2(X2,B2)</strong> - Load</p>
<p>Load 32-bit word from memory into register</p>
<p>Example: <code>L R1, 0x100</code></p>

<p><strong>ST R1, D2(X2,B2)</strong> - Store</p>
<p>Store register to memory</p>
<p>Example: <code>ST R1, 0x200</code></p>

<p><strong>LR R1, R2</strong> - Load Register</p>
<p>Copy R2 to R1</p>
<p>Example: <code>LR R1, R2</code></p>

<h4>Arithmetic Instructions</h4>
<p><strong>A R1, D2(X2,B2)</strong> - Add</p>
<p>Add memory value to register</p>
<p>Example: <code>A R1, 0x100</code></p>

<p><strong>AR R1, R2</strong> - Add Register</p>
<p>Add R2 to R1</p>
<p>Example: <code>AR R1, R2</code></p>

<p><strong>S R1, D2(X2,B2)</strong> - Subtract</p>
<p>Subtract memory value from register</p>
<p>Example: <code>S R1, 0x100</code></p>

<p><strong>SR R1, R2</strong> - Subtract Register</p>
<p>Subtract R2 from R1</p>
<p>Example: <code>SR R1, R2</code></p>

<p><strong>M R1, D2(X2,B2)</strong> - Multiply</p>
<p>Multiply register by memory value</p>
<p>Example: <code>M R1, 0x100</code></p>

<h4>Compare Instructions</h4>
<p><strong>C R1, D2(X2,B2)</strong> - Compare</p>
<p>Compare register with memory</p>
<p>Sets condition code</p>

<p><strong>CR R1, R2</strong> - Compare Register</p>
<p>Compare R1 with R2</p>
<p>Sets condition code</p>

<h4>Control Instructions</h4>
<p><strong>HIO</strong> - Halt I/O</p>
<p>Stop program execution</p>

<h4>Condition Codes:</h4>
<ul>
    <li><strong>0</strong> - Zero (equal)</li>
    <li><strong>1</strong> - Low (less than)</li>
    <li><strong>2</strong> - High (greater than)</li>
    <li><strong>3</strong> - Overflow</li>
</ul>
"#;

const HELP_CONTENT: &str = r#"
<h3>Help & Tips</h3>

<h4>How to Use:</h4>
<ol>
    <li><strong>Write Code</strong>: Enter your assembly program in the editor</li>
    <li><strong>Assemble</strong>: Click "Assemble" to convert to machine code</li>
    <li><strong>Step/Run</strong>: Use "Step" for single instructions or "Run" to complete</li>
    <li><strong>Reset</strong>: Click "Reset" to clear CPU and start over</li>
</ol>

<h4>Challenges:</h4>
<p>Click "Challenges" to see available programming puzzles. Each challenge has specific requirements and validation.</p>

<h4>Debugging Tips:</h4>
<ul>
    <li>Use "Step" to watch register values change</li>
    <li>Check the memory viewer to see your program</li>
    <li>The assembly output shows addresses and opcodes</li>
    <li>Condition codes update after comparisons</li>
</ul>

<h4>Common Mistakes:</h4>
<ul>
    <li>Forgetting to include HIO at the end</li>
    <li>Using invalid register numbers (R0-R15 only)</li>
    <li>Missing commas in instruction syntax</li>
    <li>Incorrect memory addresses</li>
</ul>

<h4>ESA/390 Conventions:</h4>
<ul>
    <li>R12 is typically used as base register</li>
    <li>R13 points to save area</li>
    <li>R14 holds return address</li>
    <li>R15 contains return code</li>
</ul>
"#;
