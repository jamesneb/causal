<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';

	let { children } = $props();
	
	// Updated navigation for Causal Systems Studio
	const navItems = [
		{ name: 'Dashboard', path: '/dashboard', icon: 'dashboard' },
		{ name: 'Services', path: '/services', icon: 'services' },
		{ name: 'Traces', path: '/traces', icon: 'trace' },
		{ name: 'Debugger', path: '/debugger', icon: 'debug' },
		{ name: 'Causal Explorer', path: '/causal', icon: 'causality' },
		{ name: 'System Synthesis', path: '/synthesis', icon: 'synthesis' },
		{ name: 'Optimizer', path: '/optimizer', icon: 'optimize' },
		{ name: 'Settings', path: '/settings', icon: 'settings' }
	];
	
	// Notifications state
	let showNotifications = false;
	let notificationCount = 3;
	let notifications = [
		{ 
			id: 1, 
			title: 'Optimization Suggestion', 
			message: 'Connection pool settings could be improved in payment-service', 
			time: '5 min ago',
			type: 'info'
		},
		{ 
			id: 2, 
			title: 'Latency Alert', 
			message: 'Database query latency exceeding threshold',
			time: '15 min ago',
			type: 'warning'
		},
		{ 
			id: 3, 
			title: 'Root Cause Detected', 
			message: 'Memory pressure identified as cause of checkout failures',
			time: '1 hour ago',
			type: 'error'
		}
	];
	
	// Close notifications when clicking outside
	function handleClickOutside(event: MouseEvent) {
		const notificationElement = document.querySelector('.notifications-dropdown');
		if (notificationElement && !notificationElement.contains(event.target as Node) && 
			!document.querySelector('.notifications-button')?.contains(event.target as Node)) {
			showNotifications = false;
		}
	}
	
	onMount(() => {
		document.addEventListener('click', handleClickOutside);
		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});
	
	// Check if path is active
	function isActive(path: string) {
		return $page.url.pathname.startsWith(path);
	}
	
	// Toggle notifications panel
	function toggleNotifications() {
		showNotifications = !showNotifications;
	}
	
	// Remove notification
	function removeNotification(id: number) {
		notifications = notifications.filter(n => n.id !== id);
		notificationCount = notifications.length;
	}
	
	// Clear all notifications
	function clearAllNotifications() {
		notifications = [];
		notificationCount = 0;
	}

	// Mobile menu toggle
	let isMobileMenuOpen = false;
	function toggleMobileMenu() {
		isMobileMenuOpen = !isMobileMenuOpen;
	}
</script>

<div class="app-container">
	<aside class="sidebar edgy-sidebar {isMobileMenuOpen ? 'mobile-open' : ''}">
		<div class="logo-container">
			<a href="/" class="logo">
				<div class="logo-icon">C</div>
				<div class="edgy-logo" data-text="CAUSAL">CAUSAL</div>
			</a>
		</div>
		
		<nav class="nav-container">
			<ul class="nav-items">
				{#each navItems as item}
					<li class="nav-item">
						<a 
							href={item.path}
							class="nav-link {isActive(item.path) ? 'active' : ''}"
							aria-current={isActive(item.path) ? 'page' : undefined}
						>
							<span class="nav-icon">
								{#if item.icon === 'dashboard'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<rect x="3" y="3" width="7" height="9" rx="1" />
										<rect x="14" y="3" width="7" height="5" rx="1" />
										<rect x="14" y="12" width="7" height="9" rx="1" />
										<rect x="3" y="16" width="7" height="5" rx="1" />
									</svg>
								{:else if item.icon === 'services'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<circle cx="12" cy="12" r="3" />
										<circle cx="19" cy="5" r="2" />
										<circle cx="5" cy="5" r="2" />
										<circle cx="5" cy="19" r="2" />
										<circle cx="19" cy="19" r="2" />
										<line x1="12" y1="9" x2="12" y2="6" />
										<line x1="15" y1="12" x2="18" y2="12" />
										<line x1="12" y1="15" x2="12" y2="18" />
										<line x1="9" y1="12" x2="6" y2="12" />
									</svg>
								{:else if item.icon === 'trace'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M3,12 h4 a2,2 0 0,1 2,2 v5 a1,1 0 0,0 1,1 h1" />
										<path d="M15,12 h4 a2,2 0 0,1 2,2 v5 a1,1 0 0,0 1,1 h1" />
										<path d="M9,4 h4 a2,2 0 0,1 2,2 v6" />
										<circle cx="7" cy="12" r="1" />
										<circle cx="19" cy="12" r="1" />
										<circle cx="11" cy="4" r="1" />
									</svg>
								{:else if item.icon === 'debug'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M12 3a3 3 0 0 0-3 3m3-3a3 3 0 0 1 3 3m-3-3v3m-9 9h3m3 0H3m6 0a6 6 0 0 1 6-6m0 0a6 6 0 0 1 6 6m0 0h3m-3 0h-3m-9 6h18" />
										<circle cx="12" cy="12" r="1" />
									</svg>
								{:else if item.icon === 'causality'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<circle cx="8" cy="8" r="3" />
										<circle cx="16" cy="16" r="3" />
										<path d="M10.5 10.5L13.5 13.5" stroke-dasharray="2" />
										<path d="M18 8H21" />
										<path d="M3 16H5" />
										<path d="M11 3V5" />
										<path d="M13 19V21" />
									</svg>
								{:else if item.icon === 'synthesis'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M12 2v2m0 16v2M4 12H2m20 0h-2m-9 7l-3-3m3-11l-3 3m12 0l-3-3m0 14l3-3" />
										<circle cx="12" cy="12" r="4" />
										<path d="M12 16V12l-3-2" stroke-dasharray="2"/>
									</svg>
								{:else if item.icon === 'optimize'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<path d="M12 2v2m0 16v2M4 12H2m20 0h-2m-9 7l-3-3m3-11l-3 3m12 0l-3-3m0 14l3-3" />
										<circle cx="12" cy="12" r="4" />
									</svg>
								{:else if item.icon === 'settings'}
									<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
										<circle cx="12" cy="12" r="3" />
										<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
									</svg>
								{/if}
							</span>
							<span class="nav-text">{item.name}</span>
						</a>
					</li>
				{/each}
			</ul>
		</nav>
		
		<div class="sidebar-footer">
			<div class="user-menu">
				<div class="user-avatar">JD</div>
				<div class="user-details">
					<div class="user-name">John Doe</div>
					<div class="user-role">Developer</div>
				</div>
			</div>
		</div>

		<!-- Mobile close button -->
		<button class="mobile-close-btn" on:click={toggleMobileMenu}>
			<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
				<line x1="18" y1="6" x2="6" y2="18" />
				<line x1="6" y1="6" x2="18" y2="18" />
			</svg>
		</button>
	</aside>
	
	<main class="content">
		<header class="content-header">
			<!-- Mobile menu button -->
			<button class="mobile-menu-btn" on:click={toggleMobileMenu}>
				<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
					<line x1="3" y1="6" x2="21" y2="6" />
					<line x1="3" y1="12" x2="21" y2="12" />
					<line x1="3" y1="18" x2="21" y2="18" />
				</svg>
			</button>

			<div class="header-title">
				{$page.url.pathname === '/' ? 'Home' : 
				 $page.url.pathname.startsWith('/dashboard') ? 'System Dashboard' : 
				 $page.url.pathname.startsWith('/services') ? 'Service Map' : 
				 $page.url.pathname.startsWith('/traces') ? 'Distributed Traces' :
				 $page.url.pathname.startsWith('/debugger') ? 'Interactive Debugger' :
				 $page.url.pathname.startsWith('/causal') ? 'Causal Explorer' :
				 $page.url.pathname.startsWith('/synthesis') ? 'System Synthesis' :
				 $page.url.pathname.startsWith('/optimizer') ? 'System Optimizer' :
				 $page.url.pathname.startsWith('/settings') ? 'Settings' : 'Causal Systems Studio'}
			</div>
			
			<div class="header-controls">
				<div class="time-selector">
					<span class="time-selector-icon">
						<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
							<circle cx="12" cy="12" r="10" />
							<polyline points="12 6 12 12 16 14" />
						</svg>
					</span>
					<span class="time-selector-text">Last 30 minutes</span>
					<span class="time-selector-arrow">▼</span>
				</div>
				
				<div class="search-box">
					<span class="search-icon">
						<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
							<circle cx="11" cy="11" r="8" />
							<line x1="21" y1="21" x2="16.65" y2="16.65" />
						</svg>
					</span>
					<input type="text" class="search-input" placeholder="Search traces, services, errors...">
				</div>
				
				<div class="notifications-container">
					<button 
						class="notifications-button" 
						type="button" 
						aria-label="Notifications"
						on:click|stopPropagation={toggleNotifications}
					>
						<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
							<path d="M13.73 21a2 2 0 0 1-3.46 0" />
						</svg>
						{#if notificationCount > 0}
							<span class="notification-badge">{notificationCount}</span>
						{/if}
					</button>
					
					{#if showNotifications}
						<div class="notifications-dropdown">
							<div class="notifications-header">
								<h3>Notifications</h3>
								{#if notifications.length > 0}
									<button class="clear-all" on:click={clearAllNotifications}>Clear All</button>
								{/if}
							</div>
							
							<div class="notifications-content">
								{#if notifications.length === 0}
									<div class="no-notifications">
										<p>No new notifications</p>
									</div>
								{:else}
									<ul class="notification-list">
										{#each notifications as notification}
											<li class="notification-item notification-{notification.type}">
												<div class="notification-icon">
													{#if notification.type === 'error'}
														<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
															<circle cx="12" cy="12" r="10" />
															<line x1="12" y1="8" x2="12" y2="12" />
															<line x1="12" y1="16" x2="12" y2="16" />
														</svg>
													{:else if notification.type === 'warning'}
														<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
															<path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
															<line x1="12" y1="9" x2="12" y2="13" />
															<line x1="12" y1="17" x2="12" y2="17" />
														</svg>
													{:else}
														<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
															<circle cx="12" cy="12" r="10" />
															<polyline points="8 12 12 16 16 12" />
															<line x1="12" y1="8" x2="12" y2="16" />
														</svg>
													{/if}
												</div>
												<div class="notification-content">
													<div class="notification-title">{notification.title}</div>
													<div class="notification-message">{notification.message}</div>
													<div class="notification-time">{notification.time}</div>
												</div>
												<button 
													class="notification-dismiss" 
													on:click|stopPropagation={() => removeNotification(notification.id)}
													aria-label="Dismiss notification"
												>
													<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
														<line x1="18" y1="6" x2="6" y2="18" />
														<line x1="6" y1="6" x2="18" y2="18" />
													</svg>
												</button>
											</li>
										{/each}
									</ul>
								{/if}
							</div>
						</div>
					{/if}
				</div>
			</div>
		</header>
		
		<div class="page-container">
			{@render children()}
		</div>
	</main>
</div>

<style>
	/* SigNoz-inspired Layout */
	.app-container {
		display: grid;
		grid-template-columns: 240px 1fr;
		min-height: 100vh;
	}
	
	/* Sidebar */
	.sidebar {
		background-color: var(--bg-darker);
		border-right: 1px solid rgba(255, 255, 255, 0.05);
		display: flex;
		flex-direction: column;
		height: 100vh;
		position: sticky;
		top: 0;
		z-index: 10;
	}
	
	.logo-container {
		padding: 16px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.05);
		margin-bottom: 16px;
	}
	
	.logo {
		display: flex;
		align-items: center;
		text-decoration: none;
	}
	
	.logo-icon {
		width: 32px;
		height: 32px;
		border-radius: 2px;
		background: linear-gradient(135deg, var(--primary), var(--accent));
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--bg-darkest);
		font-weight: 800;
		font-size: 18px;
		margin-right: 12px;
		position: relative;
		box-shadow: var(--glow-primary);
	}
	
	.logo-icon::after {
		content: '';
		position: absolute;
		top: -1px;
		left: -1px;
		right: -1px;
		bottom: -1px;
		border-radius: 2px;
		border: 1px solid var(--primary);
		opacity: 0.5;
		animation: pulse-glow 1.5s infinite alternate;
	}
	
	.nav-container {
		flex: 1;
		padding: 0;
		overflow-y: auto;
	}
	
	.nav-items {
		list-style: none;
		padding: 0;
		margin: 0;
	}
	
	.nav-item {
		margin: 0;
	}
	
	.nav-link {
		display: flex;
		align-items: center;
		padding: 10px 16px;
		color: var(--text-medium);
		text-decoration: none;
		border-left: 2px solid transparent;
		transition: all var(--transition-fast);
		position: relative;
		text-transform: uppercase;
		font-size: 13px;
		letter-spacing: 0.5px;
	}
	
	.nav-link:hover {
		background-color: var(--bg-medium);
		color: var(--text-bright);
	}
	
	.nav-link::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 0;
		background: var(--primary);
		transition: width var(--transition-normal);
	}
	
	.nav-link:hover::before {
		width: 2px;
	}
	
	.nav-link.active {
		background-color: rgba(0, 255, 170, 0.05);
		color: var(--primary);
		border-left-color: var(--primary);
	}
	
	.nav-link.active::before {
		width: 2px;
	}
	
	.nav-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		margin-right: 12px;
	}
	
	.nav-text {
		font-size: 14px;
		font-weight: 500;
	}
	
	.sidebar-footer {
		padding: 16px;
		border-top: 1px solid rgba(255, 255, 255, 0.05);
	}
	
	.user-menu {
		display: flex;
		align-items: center;
		cursor: pointer;
		padding: 8px;
		border-radius: 8px;
		transition: all 0.2s ease;
	}
	
	.user-menu:hover {
		background-color: var(--bg-medium);
	}
	
	.user-avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background-color: var(--primary);
		color: white;
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: 600;
		margin-right: 12px;
	}
	
	.user-details {
		display: flex;
		flex-direction: column;
	}
	
	.user-name {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-bright);
	}
	
	.user-role {
		font-size: 12px;
		color: var(--text-medium);
	}
	
	/* Content Area */
	.content {
		background-color: var(--bg-darkest);
	}
	
	.content-header {
		height: 60px;
		padding: 0 24px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		border-bottom: 1px solid rgba(255, 255, 255, 0.05);
		background-color: var(--bg-darker);
		position: relative;
	}
	
	.content-header::after {
		content: '';
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 1px;
		background: linear-gradient(to right, transparent, var(--primary), transparent);
		opacity: 0.3;
	}
	
	.header-title {
		font-size: 16px;
		font-weight: 700;
		color: var(--text-brightest);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		position: relative;
		padding-right: 12px;
	}
	
	.header-title::after {
		content: '';
		position: absolute;
		right: 0;
		top: 50%;
		width: 4px;
		height: 4px;
		background: var(--primary);
		transform: translateY(-50%);
		border-radius: 0;
	}
	
	.header-controls {
		display: flex;
		align-items: center;
		gap: 16px;
	}
	
	.time-selector {
		display: flex;
		align-items: center;
		background-color: var(--bg-dark);
		border-radius: 4px;
		padding: 6px 12px;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 1px solid rgba(255, 255, 255, 0.1);
	}
	
	.time-selector:hover {
		border-color: rgba(255, 255, 255, 0.2);
	}
	
	.time-selector-icon {
		margin-right: 8px;
		color: var(--text-medium);
	}
	
	.time-selector-text {
		font-weight: 500;
		font-size: 14px;
	}
	
	.time-selector-arrow {
		margin-left: 8px;
		color: var(--text-medium);
	}
	
	.search-box {
		position: relative;
	}
	
	.search-input {
		background-color: var(--bg-dark);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 4px;
		padding: 8px 12px 8px 36px;
		color: var(--text-bright);
		font-size: 14px;
		width: 240px;
		transition: all 0.2s ease;
	}
	
	.search-input:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 2px rgba(86, 97, 242, 0.2);
	}
	
	.search-icon {
		position: absolute;
		left: 12px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-medium);
	}
	
	.notifications-container {
		position: relative;
	}
	
	.notifications-button {
		background: none;
		border: 1px solid rgba(255, 255, 255, 0.1);
		color: var(--text-medium);
		width: 36px;
		height: 36px;
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all 0.2s ease;
		position: relative;
	}
	
	.notifications-button:hover {
		background-color: var(--bg-medium);
		color: var(--text-bright);
		border-color: rgba(255, 255, 255, 0.2);
	}
	
	.notification-badge {
		position: absolute;
		top: -6px;
		right: -6px;
		background-color: var(--error);
		color: white;
		font-size: 10px;
		font-weight: 600;
		min-width: 16px;
		height: 16px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0 4px;
	}
	
	.notifications-dropdown {
		position: absolute;
		top: calc(100% + 8px);
		right: 0;
		width: 320px;
		background-color: var(--bg-darker);
		border-radius: 4px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
		border: 1px solid rgba(255, 255, 255, 0.05);
		z-index: 1000;
		overflow: hidden;
	}
	
	.notifications-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 12px 16px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.05);
	}
	
	.notifications-header h3 {
		font-size: 14px;
		font-weight: 600;
		margin: 0;
		color: var(--text-bright);
	}
	
	.clear-all {
		background: none;
		border: none;
		color: var(--primary);
		font-size: 12px;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: 4px;
	}
	
	.clear-all:hover {
		background-color: rgba(86, 97, 242, 0.1);
	}
	
	.notifications-content {
		max-height: 320px;
		overflow-y: auto;
	}
	
	.no-notifications {
		display: flex;
		justify-content: center;
		align-items: center;
		padding: 24px 0;
		color: var(--text-medium);
		font-size: 14px;
	}
	
	.notification-list {
		list-style: none;
		padding: 0;
		margin: 0;
	}
	
	.notification-item {
		display: flex;
		padding: 12px 16px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.05);
		position: relative;
	}
	
	.notification-item:hover {
		background-color: var(--bg-dark);
	}
	
	.notification-item::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 3px;
	}
	
	.notification-error::before {
		background-color: var(--error);
	}
	
	.notification-warning::before {
		background-color: var(--warning);
	}
	
	.notification-info::before {
		background-color: var(--primary);
	}
	
	.notification-icon {
		flex: 0 0 24px;
		display: flex;
		align-items: flex-start;
		margin-right: 12px;
	}
	
	.notification-error .notification-icon {
		color: var(--error);
	}
	
	.notification-warning .notification-icon {
		color: var(--warning);
	}
	
	.notification-info .notification-icon {
		color: var(--primary);
	}
	
	.notification-content {
		flex: 1;
		min-width: 0;
	}
	
	.notification-title {
		font-weight: 600;
		font-size: 14px;
		margin-bottom: 4px;
		color: var(--text-bright);
	}
	
	.notification-message {
		font-size: 13px;
		color: var(--text-medium);
		margin-bottom: 4px;
	}
	
	.notification-time {
		font-size: 12px;
		color: var(--text-dim);
	}
	
	.notification-dismiss {
		background: none;
		border: none;
		color: var(--text-medium);
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 4px;
		cursor: pointer;
		opacity: 0.6;
		transition: all 0.2s ease;
		margin-left: 8px;
	}
	
	.notification-dismiss:hover {
		opacity: 1;
		background-color: rgba(255, 255, 255, 0.05);
	}
	
	.page-container {
		padding: 24px;
	}

	/* Mobile menu */
	.mobile-menu-btn, .mobile-close-btn {
		display: none;
		background: none;
		border: none;
		color: var(--text-medium);
		cursor: pointer;
		padding: 8px;
	}

	.mobile-close-btn {
		position: absolute;
		top: 16px;
		right: 16px;
	}

	/* Responsive styles */
	@media (max-width: 992px) {
		.app-container {
			grid-template-columns: 1fr;
		}

		.sidebar {
			position: fixed;
			left: -240px;
			top: 0;
			bottom: 0;
			transition: left 0.3s ease;
			box-shadow: 4px 0 10px rgba(0, 0, 0, 0.2);
		}

		.sidebar.mobile-open {
			left: 0;
		}

		.mobile-menu-btn, .mobile-close-btn {
			display: flex;
		}

		.content-header {
			padding: 0 16px;
		}

		.header-title {
			margin-left: 8px;
		}

		.header-controls {
			gap: 8px;
		}

		.time-selector {
			display: none;
		}

		.search-input {
			width: 180px;
		}

		.page-container {
			padding: 16px;
		}
	}

	@media (max-width: 576px) {
		.search-box {
			display: none;
		}
	}
</style>