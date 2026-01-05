import { useState } from 'react'
import type { Feed } from '../../types'
import FeedItem from './FeedItem'
import AddFeedModal from './AddFeedModal'
import Button from '../ui/Button'

interface FeedListProps {
  feeds: Feed[]
  selectedFeedId: number | null
  onSelectFeed: (id: number | null) => void
  onAddFeed: (url: string) => Promise<Feed>
  onDeleteFeed: (id: number) => Promise<void>
  onRenameFeed: (id: number, title: string) => Promise<void>
  onRefreshFeed: (id: number) => Promise<number>
  onRefreshAll: () => Promise<void>
}

export default function FeedList({
  feeds,
  selectedFeedId,
  onSelectFeed,
  onAddFeed,
  onDeleteFeed,
  onRenameFeed,
  onRefreshFeed,
  onRefreshAll,
}: FeedListProps) {
  const [showAddModal, setShowAddModal] = useState(false)
  const [refreshing, setRefreshing] = useState(false)

  const handleAdd = async (url: string) => {
    await onAddFeed(url)
  }

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this feed?')) {
      await onDeleteFeed(id)
    }
  }

  const handleRefreshAll = async () => {
    try {
      setRefreshing(true)
      await onRefreshAll()
    } finally {
      setRefreshing(false)
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-3 border-b border-gray-200">
        <div className="flex items-center justify-between mb-2">
          <h2 className="font-semibold text-gray-900">Feeds</h2>
          <button
            onClick={handleRefreshAll}
            disabled={refreshing}
            className="p-1 hover:bg-gray-100 rounded transition-colors disabled:opacity-50"
            title="Refresh all feeds"
          >
            <svg
              className={`w-4 h-4 text-gray-600 ${refreshing ? 'animate-spin' : ''}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
          </button>
        </div>

        {/* All Articles Button */}
        <button
          onClick={() => onSelectFeed(null)}
          className={`w-full px-3 py-2 text-left text-sm rounded-md transition-colors ${
            selectedFeedId === null
              ? 'bg-orange-100 text-orange-900 font-medium'
              : 'hover:bg-gray-100'
          }`}
        >
          All Articles
        </button>
      </div>

      {/* Feed List */}
      <div className="flex-1 overflow-y-auto">
        {feeds.length === 0 ? (
          <div className="p-4 text-center text-gray-500 text-sm">
            No feeds yet. Click + to add one.
          </div>
        ) : (
          <div className="py-1">
            {feeds.map((feed) => (
              <FeedItem
                key={feed.id}
                feed={feed}
                isSelected={selectedFeedId === feed.id}
                onSelect={() => onSelectFeed(feed.id)}
                onDelete={() => handleDelete(feed.id)}
                onRename={(title) => onRenameFeed(feed.id, title)}
                onRefresh={() => onRefreshFeed(feed.id)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Add Button */}
      <div className="p-3 border-t border-gray-200">
        <Button
          onClick={() => setShowAddModal(true)}
          className="w-full"
          size="sm"
        >
          <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          Add Feed
        </Button>
      </div>

      <AddFeedModal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onAdd={handleAdd}
      />
    </div>
  )
}
