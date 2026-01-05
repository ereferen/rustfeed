import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Feed } from '../types'

interface UseFeedsReturn {
  feeds: Feed[]
  loading: boolean
  error: string | null
  selectedFeedId: number | null
  selectFeed: (id: number | null) => void
  addFeed: (url: string) => Promise<Feed>
  deleteFeed: (id: number) => Promise<void>
  renameFeed: (id: number, title: string) => Promise<void>
  refreshFeed: (id: number) => Promise<number>
  refreshAllFeeds: () => Promise<void>
  reload: () => Promise<void>
}

export function useFeeds(): UseFeedsReturn {
  const [feeds, setFeeds] = useState<Feed[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedFeedId, setSelectedFeedId] = useState<number | null>(null)

  const loadFeeds = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const feedList = await invoke<Feed[]>('get_feeds')
      setFeeds(feedList)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadFeeds()
  }, [loadFeeds])

  const selectFeed = useCallback((id: number | null) => {
    setSelectedFeedId(id)
  }, [])

  const addFeed = useCallback(async (url: string): Promise<Feed> => {
    const feed = await invoke<Feed>('add_feed', { url })
    setFeeds(prev => [...prev, feed])
    return feed
  }, [])

  const deleteFeed = useCallback(async (id: number): Promise<void> => {
    await invoke('delete_feed', { id })
    setFeeds(prev => prev.filter(f => f.id !== id))
    if (selectedFeedId === id) {
      setSelectedFeedId(null)
    }
  }, [selectedFeedId])

  const renameFeed = useCallback(async (id: number, title: string): Promise<void> => {
    await invoke('rename_feed', { id, title })
    setFeeds(prev => prev.map(f => f.id === id ? { ...f, title } : f))
  }, [])

  const refreshFeed = useCallback(async (id: number): Promise<number> => {
    const count = await invoke<number>('fetch_feed', { id })
    return count
  }, [])

  const refreshAllFeeds = useCallback(async (): Promise<void> => {
    await invoke('fetch_all_feeds')
    await loadFeeds()
  }, [loadFeeds])

  return {
    feeds,
    loading,
    error,
    selectedFeedId,
    selectFeed,
    addFeed,
    deleteFeed,
    renameFeed,
    refreshFeed,
    refreshAllFeeds,
    reload: loadFeeds,
  }
}
