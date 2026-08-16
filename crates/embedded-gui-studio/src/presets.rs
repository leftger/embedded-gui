//! Built-in declarative KDL screen presets for Embedded GUI Studio.

pub const SAMPLE_AUTOMOTIVE_CLUSTER: &str = r#"screen id="AutoCluster" width=480 height=272 theme="dark" {
    grid cols="1fr 120px 1fr" rows="32px 1fr 40px" gap=6 padding=8 {
        status_bar id="top_bar" time="10:42" col=0 row=0 col_span=3
        scale id="tachometer" mode="radial" min=0.0 max=8000.0 value=4500.0 major_ticks=8 col=0 row=1
        label id="gear_speed" text="D4 68 MPH" style="bold" col=1 row=1
        scale id="speedometer" mode="radial" min=0.0 max=160.0 value=68.0 major_ticks=8 col=2 row=1
        progress id="fuel_level" value=0.72 col=0 row=2
        toggle id="sport_mode" label="SPORT" checked=true col=1 row=2
        progress id="temp_gauge" value=0.48 col=2 row=2
    }
}
"#;

pub const SAMPLE_HVAC_CLIMATE: &str = r#"screen id="HvacClimate" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="32px 1fr 48px" gap=6 padding=6 {
        status_bar id="hvac_status" time="15:30" col=0 row=0 col_span=2
        scale id="ambient_temp" mode="radial" min=15.0 max=35.0 value=21.5 major_ticks=4 col=0 row=1
        slider id="target_slider" min=16 max=30 value=22 col=1 row=1
        roller id="mode_picker" selected=2 col=0 row=2 {
            option "OFF"
            option "ECO"
            option "AUTO"
            option "COOL"
            option "HEAT"
        }
        busy_wheel id="fan_spinner" active=true col=1 row=2
    }
}
"#;

pub const SAMPLE_PATIENT_MONITOR: &str = r#"screen id="PatientMonitor" width=480 height=272 theme="dark" {
    grid cols="1fr 140px" rows="32px 1fr 1fr" gap=6 padding=6 {
        status_bar id="bed_header" time="08:15" col=0 row=0 col_span=2
        plotter id="ecg_lead_ii" mode="sine" col=0 row=1
        label id="hr_readout" text="HR: 72 BPM" style="accent" col=1 row=1
        plotter id="spo2_pleth" mode="triangle" col=0 row=2
        label id="bp_readout" text="BP: 120/80" style="bold" col=1 row=2
    }
}
"#;

pub const SAMPLE_CNC_CONTROLLER: &str = r#"screen id="CncController" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="30px 1fr 40px" gap=4 padding=6 {
        label id="cnc_title" text="CNC MILLING AXIS CONTROLLER" style="bold" col=0 row=0 col_span=2
        scale id="spindle_rpm" mode="radial" min=0.0 max=24000.0 value=12000.0 major_ticks=6 col=0 row=1
        slider id="feed_override" min=10 max=150 value=100 col=1 row=1
        button id="btn_estop" text="EMERGENCY STOP" style="danger" col=0 row=2
        toggle id="cycle_start" label="CYCLE RUN" checked=true col=1 row=2
    }
}
"#;

pub const SAMPLE_SMARTWATCH_FITNESS: &str = r#"screen id="FitnessTracker" width=240 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="28px 1fr 42px" gap=4 padding=6 {
        status_bar id="watch_bar" time="17:45" col=0 row=0 col_span=2
        scale id="move_ring" mode="radial" min=0.0 max=600.0 value=480.0 major_ticks=6 col=0 row=1
        plotter id="live_heart" mode="sine" col=1 row=1
        progress id="daily_goal" value=0.80 col=0 row=2
        button id="btn_workout" text="START WORKOUT" style="accent" col=1 row=2
    }
}
"#;

pub const SAMPLE_THERMOSTAT: &str = r#"screen id="Thermostat" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="36px 1fr 48px" gap=6 padding=8 {
        status_bar id="status" time="14:32" col=0 row=0 col_span=2
        scale id="temp_gauge" mode="radial" min=15.0 max=35.0 value=22.5 major_ticks=4 col=0 row=1
        slider id="target_slider" min=10 max=40 value=23 col=1 row=1
        button id="btn_heat" text="Heat Mode" style="accent" col=0 row=2
        toggle id="power_switch" label="Power" checked=true col=1 row=2
    }
}
"#;

pub const SAMPLE_DASHBOARD: &str = r#"screen id="SensorDashboard" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="32px 1fr 1fr" gap=4 padding=6 {
        label id="header" text="ENVIRONMENTAL TELEMETRY" style="bold" col=0 row=0 col_span=2
        panel id="pnl_temp" style="card" col=0 row=1
        panel id="pnl_hum" style="card" col=1 row=1
        progress id="battery" value=0.85 col=0 row=2
        button id="btn_sync" text="Sync Telemetry" col=1 row=2
    }
}
"#;

pub const SAMPLE_WAVEFORM: &str = r#"screen id="ScopeScreen" width=320 height=240 theme="dark" {
    grid cols="1fr 80px" rows="30px 1fr 40px" gap=4 padding=4 {
        label id="title" text="DSO-X 2-CHANNEL OSCILLOSCOPE" col=0 row=0 col_span=2
        plotter id="wave_view" mode="sine" col=0 row=1
        roller id="v_div" selected=1 col=1 row=1 {
            option "100mV"
            option "500mV"
            option "1V"
            option "5V"
        }
        button id="btn_run" text="RUN/STOP" style="accent" col=0 row=2
        button id="btn_single" text="SINGLE" col=1 row=2
    }
}
"#;

pub const SAMPLE_MOTION_KITCHEN_SINK: &str = r#"screen id="MotionShowcase" width=320 height=240 theme="dark" {
    grid cols="1fr 1fr" rows="32px 1fr 1fr" gap=6 padding=6 {
        status_bar id="clock" time="12:00" col=0 row=0 col_span=2
        plotter id="live_scope" mode="sine" col=0 row=1
        busy_wheel id="spinner" active=true col=1 row=1
        progress id="pulse_bar" value=0.5 col=0 row=2
        scale id="dyn_gauge" mode="radial" min=0.0 max=100.0 value=50.0 major_ticks=5 col=1 row=2
    }
}
"#;
