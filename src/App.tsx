function App() {
  return (
    <main
      style={{
        minHeight: "100vh",
        display: "grid",
        placeItems: "center",
        margin: 0,
        fontFamily:
          'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
        background: "#101820",
        color: "#f6f8fa",
      }}
    >
      <section style={{ textAlign: "center" }}>
        <h1 style={{ margin: "0 0 12px", fontSize: "32px", fontWeight: 700 }}>
          Codex Token Monitor
        </h1>
        <p style={{ margin: 0, color: "#b8c3cf" }}>Tauri + React + TypeScript</p>
      </section>
    </main>
  );
}

export default App;
