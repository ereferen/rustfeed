import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface Feed {
  id: number
  url: string
  title: string
  description: string | null
  created_at: string
  updated_at: string
}

function App() {
  const [feeds, setFeeds] = useState<Feed[]>([])
  const [version, setVersion] = useState<string>('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function loadData() {
      try {
        // バージョン取得
        const ver = await invoke<string>('get_app_version')
        setVersion(ver)

        // フィード一覧取得
        const feedList = await invoke<Feed[]>('get_feeds')
        setFeeds(feedList)
      } catch (e) {
        setError(String(e))
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-gray-600">Loading...</div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-orange-600 text-white shadow-lg">
        <div className="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
          <h1 className="text-2xl font-bold">rustfeed</h1>
          <span className="text-sm opacity-75">v{version}</span>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8">
        {error && (
          <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
            {error}
          </div>
        )}

        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-xl font-semibold mb-4">Registered Feeds</h2>

          {feeds.length === 0 ? (
            <p className="text-gray-500">No feeds registered yet.</p>
          ) : (
            <ul className="divide-y divide-gray-200">
              {feeds.map((feed) => (
                <li key={feed.id} className="py-4">
                  <div className="flex justify-between items-start">
                    <div>
                      <h3 className="font-medium text-gray-900">{feed.title}</h3>
                      <p className="text-sm text-gray-500 truncate max-w-xl">{feed.url}</p>
                      {feed.description && (
                        <p className="text-sm text-gray-600 mt-1">{feed.description}</p>
                      )}
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* Status */}
        <div className="mt-8 text-center text-gray-500 text-sm">
          <p>Tauri + React + TypeScript + Tailwind CSS</p>
          <p className="mt-1">Connected to rustfeed-core via Tauri Commands</p>
        </div>
      </main>
    </div>
  )
}

export default App
