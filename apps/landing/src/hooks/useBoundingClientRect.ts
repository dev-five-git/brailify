'use client'

import { useEffect, useState } from 'react'

export const useBoundingClientRect = (
  ref: React.RefObject<HTMLElement | null>,
) => {
  const [rect, setRect] = useState<DOMRect | null>(null)

  useEffect(() => {
    if (ref.current) {
      setRect(ref.current.getBoundingClientRect())
    }
  }, [ref])

  return rect
}
