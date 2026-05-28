<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { withBase } from 'vitepress'

const canvasRef = ref<HTMLCanvasElement | null>(null)
let cleanup: (() => void) | undefined

onMounted(async () => {
  const THREE = await import('three')
  const canvas = canvasRef.value
  if (!canvas) return
  const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  const narrow = window.matchMedia('(max-width: 720px)').matches
  if (reduce || narrow) return

  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true })
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.7))
  const scene = new THREE.Scene()
  const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100)
  camera.position.set(0, 2.2, 8)

  const terminal = new THREE.Mesh(
    new THREE.BoxGeometry(3.7, 2.15, .16),
    new THREE.MeshStandardMaterial({ color: 0x08111b, emissive: 0x072436, emissiveIntensity: .85, metalness: .35, roughness: .42 })
  )
  scene.add(terminal)

  const edge = new THREE.LineSegments(
    new THREE.EdgesGeometry(terminal.geometry),
    new THREE.LineBasicMaterial({ color: 0x8ce7ff, transparent: true, opacity: .48 })
  )
  terminal.add(edge)

  const nodeMaterial = new THREE.MeshStandardMaterial({ color: 0x0d1c26, emissive: 0x0d5265, emissiveIntensity: .75, metalness: .2, roughness: .3 })
  const statusMaterial = new THREE.MeshBasicMaterial({ color: 0x5ef2a0 })
  const nodes: any[] = []
  const positions = [
    [-3.2, 1.75, -.45], [3.0, 1.55, -.2], [-3.0, -1.45, .12], [3.18, -1.15, -.38], [0, -2.55, .28]
  ]
  for (const [x, y, z] of positions) {
    const n = new THREE.Mesh(new THREE.SphereGeometry(.22, 24, 24), nodeMaterial)
    n.position.set(x, y, z)
    nodes.push(n)
    scene.add(n)
    const dot = new THREE.Mesh(new THREE.SphereGeometry(.055, 12, 12), statusMaterial)
    dot.position.set(x + .18, y + .12, z + .08)
    scene.add(dot)
  }

  const lineMat = new THREE.LineBasicMaterial({ color: 0x8ce7ff, transparent: true, opacity: .5 })
  const lines: any[] = []
  for (const n of nodes) {
    const pts = [new THREE.Vector3(0, 0, 0), n.position.clone()]
    const line = new THREE.Line(new THREE.BufferGeometry().setFromPoints(pts), lineMat)
    lines.push(line)
    scene.add(line)
  }

  const panels: any[] = []
  const panelMat = new THREE.MeshStandardMaterial({ color: 0x0b1720, emissive: 0x061c28, emissiveIntensity: .5, metalness: .15, roughness: .42 })
  const panelGeo = new THREE.BoxGeometry(.95, .38, .08)
  for (let i = 0; i < 4; i++) {
    const p = new THREE.Mesh(panelGeo, panelMat)
    p.position.set(-1.32 + i * .88, -1.42, .16)
    panels.push(p)
    scene.add(p)
  }

  const particles: any[] = []
  const particleMat = new THREE.MeshBasicMaterial({ color: 0x5ef2a0 })
  const particleGeo = new THREE.SphereGeometry(.045, 10, 10)
  for (let i = 0; i < 10; i++) {
    const m = new THREE.Mesh(particleGeo, particleMat)
    particles.push(m)
    scene.add(m)
  }

  const glow = new THREE.PointLight(0x8ce7ff, 2.4, 12)
  glow.position.set(0, 1.5, 4)
  scene.add(glow)
  scene.add(new THREE.AmbientLight(0x83b6c7, .72))

  const resize = () => {
    const parent = canvas.parentElement
    const width = parent?.clientWidth || 600
    const height = parent?.clientHeight || 520
    renderer.setSize(width, height, false)
    camera.aspect = width / height
    camera.updateProjectionMatrix()
  }
  resize()
  window.addEventListener('resize', resize)

  let frame = 0
  let raf = 0
  const animate = () => {
    frame += .01
    scene.rotation.y = Math.sin(frame * .55) * .12
    scene.rotation.x = Math.sin(frame * .38) * .045
    nodes.forEach((n, i) => {
      n.position.y += Math.sin(frame * 2 + i) * .0009
    })
    particles.forEach((p, i) => {
      const route = i % nodes.length
      const target = nodes[route].position
      const t = (frame * (.28 + route * .018) + i * .13) % 1
      p.position.lerpVectors(new THREE.Vector3(0, 0, .05), target, t)
      p.material.opacity = Math.sin(t * Math.PI)
    })
    panels.forEach((p, i) => {
      p.position.z = .16 + Math.sin(frame * 2.2 + i) * .025
    })
    renderer.render(scene, camera)
    raf = requestAnimationFrame(animate)
  }
  animate()

  cleanup = () => {
    cancelAnimationFrame(raf)
    window.removeEventListener('resize', resize)
    renderer.dispose()
    terminal.geometry.dispose()
    panelGeo.dispose()
    particleGeo.dispose()
  }
})

onBeforeUnmount(() => cleanup?.())
</script>

<template>
  <main class="sshdeck-landing">
    <nav class="deck-nav" aria-label="SSHDeck site navigation">
      <a class="deck-brand" href="/sshdeck/">
        <img :src="withBase('/logo.svg')" alt="SSHDeck logo">
        <span>SSHDeck</span>
      </a>
      <div class="deck-links">
        <a href="/sshdeck/docs/getting-started">Docs</a>
        <a href="/sshdeck/docs/sshdeck-files">Files</a>
        <a href="/sshdeck/docs/security">Security</a>
        <a href="https://github.com/PLASMA-FR/sshdeck">GitHub</a>
      </div>
    </nav>

    <section class="deck-hero">
      <div>
        <h1>SSHDeck <span>Termius + Yazi for your terminal.</span></h1>
        <p>A clean, local-first SSH command center built in Rust. No cloud. No account. No Electron. Just your terminal and OpenSSH.</p>
        <div class="deck-actions">
          <a class="deck-button primary" href="/sshdeck/docs/getting-started">Get Started</a>
          <a class="deck-button secondary" href="https://github.com/PLASMA-FR/sshdeck">View on GitHub</a>
          <a class="deck-button secondary" href="/sshdeck/docs/installation">Install</a>
        </div>
        <div class="deck-install"><span>$</span><code>git clone https://github.com/PLASMA-FR/sshdeck && cd sshdeck && cargo install --path .</code></div>
        <div class="deck-badges" aria-label="Technology badges">
          <span class="deck-badge">Rust</span>
          <span class="deck-badge">Ratatui</span>
          <span class="deck-badge">OpenSSH</span>
          <span class="deck-badge">Local-first</span>
          <span class="deck-badge">Mouse support</span>
          <span class="deck-badge">Yazi-style files</span>
        </div>
      </div>

      <div class="deck-scene-card" aria-label="3D SSH command center animation">
        <canvas ref="canvasRef"></canvas>
        <div class="scene-fallback">
          <div class="scene-terminal">
            <strong>SSHDeck</strong>
            <div class="scene-row"></div>
            <div class="scene-row mid"></div>
            <div class="scene-row short"></div>
          </div>
        </div>
        <div class="scene-label">
          Manage every SSH workflow from one terminal.
          <small>Hosts. Files. Tunnels. Commands. Health.</small>
        </div>
      </div>
    </section>

    <section class="deck-section">
      <h2>Why SSHDeck?</h2>
      <p>SSHDeck brings GUI-level SSH organization back into the terminal. It reads OpenSSH config, stores only local metadata, and gives servers a fast command-center workflow.</p>
      <div class="deck-grid">
        <article class="deck-card wide accent"><h3>Termius-like convenience</h3><p>Hosts, favorites, groups, details, command palette, tunnels, and logs are organized in one local terminal app.</p></article>
        <article class="deck-card"><h3>Yazi-style files</h3><p>Remote browsing is designed around columns, preview, selection, bookmarks, and a transfer queue model.</p></article>
        <article class="deck-card"><h3>OpenSSH-native</h3><p>SSHDeck uses system ssh for connections instead of replacing your trusted SSH stack.</p></article>
        <article class="deck-card"><h3>Keyboard and mouse</h3><p>Vim-style movement, command palette access, context menus, buttons, and scrollable panels can work together.</p></article>
        <article class="deck-card"><h3>Rust-powered TUI</h3><p>Built with ratatui and crossterm for a fast local interface with clean terminal behavior.</p></article>
      </div>
    </section>

    <section class="deck-section">
      <h2>A command center, not a host list.</h2>
      <p>The MVP already includes a polished dashboard shell, managed host flows, mouse regions, search, command generation, logs, and a Files prototype. Incomplete execution workflows are documented honestly.</p>
      <div class="deck-mockup" aria-label="SSHDeck dashboard terminal mockup">
        <div class="mock-head"><span>▣ SSHDeck</span><span>Local-first | OpenSSH</span></div>
        <div class="mock-body">
          <div class="mock-col"><div class="mock-line active"><span>All hosts</span><span>12</span></div><div class="mock-line"><span>Favorites</span><span>4</span></div><div class="mock-line"><span>Production</span><span>5</span></div><div class="mock-line"><span>Homelab</span><span>3</span></div></div>
          <div class="mock-col"><div class="mock-line active"><span>web-prod-1</span><span>root@22</span></div><div class="mock-line"><span>db-prod-1</span><span>admin@22</span></div><div class="mock-line"><span>nas</span><span>ahmad@22</span></div><div class="mock-line"><span>pi-server</span><span>pi@22</span></div></div>
          <div class="mock-col"><div class="mock-line"><span>Connect</span><span>Enter</span></div><div class="mock-line"><span>Files</span><span>s</span></div><div class="mock-line"><span>Tunnel</span><span>t</span></div><div class="mock-line"><span>Health</span><span>h</span></div></div>
        </div>
        <div class="mock-foot"><span>/ search</span><span>Ctrl+p palette</span><span>? help</span></div>
      </div>
    </section>

    <section class="deck-section">
      <h2>SSHDeck Files.</h2>
      <p>A Yazi-inspired remote file workflow is one of the headline design goals. Today it is a prototype with remote listing, preview metadata, hidden-file toggle, breadcrumbs, and transfer queue UI.</p>
      <div class="deck-grid">
        <article class="deck-card tall accent"><h3>Three-column navigation</h3><p>Parent, current directory, and preview panels keep remote context visible without leaving the keyboard.</p></article>
        <article class="deck-card"><h3>Dual-pane direction</h3><p>The local and remote split view is present as a UI placeholder while the backing file model is being completed.</p></article>
        <article class="deck-card"><h3>Transfer queue model</h3><p>Uploads, downloads, pending work, completion, failure, retry, and animation states are represented in code.</p></article>
        <article class="deck-card"><h3>Safety-first previews</h3><p>Sensitive path helpers block risky preview flows before remote commands are invoked.</p></article>
      </div>
    </section>

    <section class="deck-section deck-safety">
      <div>
        <h2>Built around local trust.</h2>
        <p>SSHDeck is intentionally boring where it matters: it does not run a cloud service, require an account, phone home, or take ownership of your SSH identity.</p>
      </div>
      <div class="deck-callout">
        <div class="deck-steps">
          <div class="deck-step">Uses your existing OpenSSH tools for v1 connection launching.</div>
          <div class="deck-step">Writes SSHDeck-managed hosts to a separate config file.</div>
          <div class="deck-step">Creates backups before config include-line changes.</div>
          <div class="deck-step">Warns or blocks dangerous command patterns in command helpers.</div>
          <div class="deck-step">Redacts known sensitive path markers from local logs.</div>
        </div>
      </div>
    </section>

    <section class="deck-section">
      <h2>Install from source.</h2>
      <p>SSHDeck is not published on crates.io yet. Use the source install flow until packaged releases are available.</p>
      <div class="deck-mockup"><div class="mock-head"><span>Install</span><span>Rust required</span></div><pre style="margin:0;padding:1.2rem;overflow:auto;color:#d8f4fb"><code>git clone https://github.com/PLASMA-FR/sshdeck
cd sshdeck
cargo install --path .
sshdeck
sshdeck doctor
sshdeck import</code></pre></div>
      <div class="deck-actions"><a class="deck-button primary" href="/sshdeck/docs/quickstart">Read the docs</a><a class="deck-button secondary" href="/sshdeck/docs/roadmap">View roadmap</a></div>
    </section>

    <footer class="deck-footer">
      <span>SSHDeck is open source under MIT.</span>
      <span><a href="https://github.com/PLASMA-FR/sshdeck">GitHub</a> / <a href="/sshdeck/docs/">Docs</a> / <a href="/sshdeck/docs/contributing">Contributing</a></span>
    </footer>
  </main>
</template>
