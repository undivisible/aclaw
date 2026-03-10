#!/usr/bin/env -S v run
// Reminder system for pending credentials
// V 0.5.1 compatible

import time
import os

const reminder_interval = 3 * time.hour // Every 3 hours
const state_file = '/tmp/openclaw_reminder_state.txt'

struct ReminderState {
mut:
	notion_creds_received bool
	gcal_creds_received   bool
	perplexity_link_received bool
	last_reminded time.Time
}

fn load_state() !ReminderState {
	if !os.exists(state_file) {
		return ReminderState{
			last_reminded: time.now()
		}
	}
	
	content := os.read_file(state_file)!
	lines := content.split('\n')
	
	mut state := ReminderState{}
	for line in lines {
		parts := line.split('=')
		if parts.len != 2 { continue }
		
		match parts[0] {
			'notion_creds' { state.notion_creds_received = parts[1] == 'true' }
			'gcal_creds' { state.gcal_creds_received = parts[1] == 'true' }
			'perplexity_link' { state.perplexity_link_received = parts[1] == 'true' }
			'last_reminded' { 
				timestamp := parts[1].i64()
				state.last_reminded = time.unix(timestamp)
			}
			else {}
		}
	}
	
	return state
}

fn save_state(state ReminderState) ! {
	mut content := ''
	content += 'notion_creds=${state.notion_creds_received}\n'
	content += 'gcal_creds=${state.gcal_creds_received}\n'
	content += 'perplexity_link=${state.perplexity_link_received}\n'
	content += 'last_reminded=${state.last_reminded.unix()}\n'
	
	os.write_file(state_file, content)!
}

fn check_reminders() ! {
	mut state := load_state()!
	
	now := time.now()
	elapsed := now - state.last_reminded
	
	if elapsed < reminder_interval {
		println('Next reminder in ${(reminder_interval - elapsed).minutes()} minutes')
		return
	}
	
	// Check what's still pending
	mut pending := []string{}
	
	if !state.notion_creds_received {
		pending << '📝 Notion credentials'
	}
	if !state.gcal_creds_received {
		pending << '📅 Google Calendar credentials'
	}
	if !state.perplexity_link_received {
		pending << '🔍 Perplexity search link'
	}
	
	if pending.len > 0 {
		println('\n🔔 REMINDER - Still waiting for:')
		for item in pending {
			println('  • ${item}')
		}
		println('')
		
		state.last_reminded = now
		save_state(state)!
	} else {
		println('✅ All credentials received!')
	}
}

fn mark_received(item string) ! {
	mut state := load_state()!
	
	match item {
		'notion' { state.notion_creds_received = true }
		'gcal' { state.gcal_creds_received = true }
		'perplexity' { state.perplexity_link_received = true }
		else { return error('Unknown item: ${item}') }
	}
	
	save_state(state)!
	println('✓ Marked ${item} as received')
}

fn main() {
	if os.args.len > 1 {
		match os.args[1] {
			'check' { check_reminders() or { panic(err) } }
			'mark' {
				if os.args.len < 3 {
					eprintln('Usage: reminder_system.v mark [notion|gcal|perplexity]')
					exit(1)
				}
				mark_received(os.args[2]) or { panic(err) }
			}
			'reset' {
				os.rm(state_file) or {}
				println('✓ Reminder state reset')
			}
			else {
				eprintln('Usage: reminder_system.v [check|mark|reset]')
				exit(1)
			}
		}
	} else {
		check_reminders() or { panic(err) }
	}
}
