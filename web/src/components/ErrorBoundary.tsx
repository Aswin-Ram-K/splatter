import React, { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryProps {
	children: ReactNode;
}

interface ErrorBoundaryState {
	hasError: boolean;
	error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
	constructor(props: ErrorBoundaryProps) {
		super(props);
		this.state = { hasError: false, error: null };
	}

	static getDerivedStateFromError(error: Error): ErrorBoundaryState {
		return { hasError: true, error };
	}

	componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
		console.error("[ErrorBoundary] Caught error:", error);
		console.error("[ErrorBoundary] Component stack:", errorInfo.componentStack);
	}

	render(): ReactNode {
		if (this.state.hasError && this.state.error) {
			return (
				<div
					style={{
						height: "100vh",
						width: "100vw",
						display: "flex",
						flexDirection: "column",
						alignItems: "center",
						justifyContent: "center",
						background: "#1a1b26",
						color: "#f7768e",
						fontFamily: "monospace",
						padding: "2rem",
						overflow: "auto",
					}}
				>
					<h2 style={{ marginBottom: "1rem" }}>Splatter — Application Error</h2>
					<pre
						style={{
							background: "#242839",
							padding: "1rem",
							borderRadius: "8px",
							maxWidth: "800px",
							maxHeight: "60vh",
							overflow: "auto",
							fontSize: "12px",
							lineHeight: "1.5",
							color: "#a9b1d6",
							width: "100%",
						}}
					>
						{this.state.error?.stack || String(this.state.error)}
					</pre>
					<p
						style={{
							marginTop: "1rem",
							color: "#7aa2f7",
							fontSize: "14px",
						}}
					>
						Check browser console (F12) for full details
					</p>
				</div>
			);
		}
		return this.props.children;
	}
}
