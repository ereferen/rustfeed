import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import FeedList from './components/feed/FeedList'
import { useFeeds } from './hooks/useFeeds'
import type { Article } from './types'

function App() {
  const [version, setVersion] = useState<string>('')
  const [articles, setArticles] = useState<Article[]>([])
  const [articlesLoading, setArticlesLoading] = useState(false)

  const {
    feeds,
    loading: feedsLoading,
    error,
    selectedFeedId,
    selectFeed,
    addFeed,
    deleteFeed,
    renameFeed,
    refreshFeed,
    refreshAllFeeds,
  } = useFeeds()

  // Load version
  useEffect(() => {
    invoke<string>('get_app_version').then(setVersion)
  }, [])

  // Load articles when selected feed changes
  useEffect(() => {
    async function loadArticles() {
      setArticlesLoading(true)
      try {
        const articleList = await invoke<Article[]>('get_articles', {
          feedId: selectedFeedId,
          unreadOnly: false,
          favoritesOnly: false,
          search: null,
          limit: 100,
        })
        setArticles(articleList)
      } catch (e) {
        console.error('Failed to load articles:', e)
      } finally {
        setArticlesLoading(false)
      }
    }
    loadArticles()
  }, [selectedFeedId])

  if (feedsLoading) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-gray-600">Loading...</div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-100 flex flex-col">
      {/* Header */}
      <header className="bg-orange-600 text-white shadow-lg shrink-0">
        <div className="px-4 py-3 flex justify-between items-center">
          <h1 className="text-xl font-bold">rustfeed</h1>
          <span className="text-sm opacity-75">v{version}</span>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        <aside className="w-60 bg-white border-r border-gray-200 shrink-0">
          <FeedList
            feeds={feeds}
            selectedFeedId={selectedFeedId}
            onSelectFeed={selectFeed}
            onAddFeed={addFeed}
            onDeleteFeed={deleteFeed}
            onRenameFeed={renameFeed}
            onRefreshFeed={refreshFeed}
            onRefreshAll={refreshAllFeeds}
          />
        </aside>

        {/* Article List */}
        <main className="flex-1 overflow-y-auto p-4">
          {error && (
            <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
              {error}
            </div>
          )}

          <div className="bg-white rounded-lg shadow">
            <div className="px-4 py-3 border-b border-gray-200">
              <h2 className="font-semibold text-gray-900">
                {selectedFeedId
                  ? feeds.find((f) => f.id === selectedFeedId)?.title || 'Articles'
                  : 'All Articles'}
              </h2>
            </div>

            {articlesLoading ? (
              <div className="p-8 text-center text-gray-500">Loading articles...</div>
            ) : articles.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                No articles yet. Add a feed and refresh to get started.
              </div>
            ) : (
              <ul className="divide-y divide-gray-200">
                {articles.map((article) => (
                  <li
                    key={article.id}
                    className={`px-4 py-3 hover:bg-gray-50 cursor-pointer ${
                      article.is_read ? 'opacity-60' : ''
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      <div className="flex-1 min-w-0">
                        <h3
                          className={`text-sm truncate ${
                            article.is_read ? 'font-normal' : 'font-medium'
                          }`}
                        >
                          {article.title}
                        </h3>
                        <p className="text-xs text-gray-500 truncate mt-0.5">
                          {article.url}
                        </p>
                        {article.published_at && (
                          <p className="text-xs text-gray-400 mt-1">
                            {new Date(article.published_at).toLocaleDateString()}
                          </p>
                        )}
                      </div>
                      {article.is_favorite && (
                        <span className="text-yellow-500">
                          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                          </svg>
                        </span>
                      )}
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </main>
      </div>
    </div>
  )
}

export default App
