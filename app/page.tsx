export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-zinc-950 text-zinc-100">
      <div className="border border-zinc-800 p-12 rounded-2xl bg-zinc-900/50 backdrop-blur-md">
        <h1 className="text-5xl font-light tracking-widest uppercase">
          Mambo<span className="font-bold text-orange-500">Site</span>
        </h1>
        <div className="mt-6 flex items-center gap-4">
          <span className="relative flex h-3 w-3">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-orange-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-3 w-3 bg-orange-500"></span>
          </span>
          <p className="font-mono text-xs text-zinc-500 uppercase tracking-tighter">
            System Online // Development in Progress
          </p>
        </div>
      </div>
    </main>
  );
}