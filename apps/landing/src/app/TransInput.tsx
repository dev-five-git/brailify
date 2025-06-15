'use client'

import { Box, DevupProps, Flex, Input, Text } from '@devup-ui/react'
import { useEffect, useRef, useState } from 'react'

import { Merge } from '@/types'

export function TransInput({
  blurPlaceholder,
  focusPlaceholder,
  isFocused,
  ...props
}: {
  blurPlaceholder: string
  focusPlaceholder: string
  isFocused?: boolean
} & Merge<React.ComponentProps<'textarea'>, DevupProps<'textarea'>>) {
  const [boxHeight, setBoxHeight] = useState(0)
  const placeholderRef = useRef<HTMLSpanElement>(null)

  useEffect(() => {
    if (placeholderRef.current) {
      setBoxHeight(placeholderRef.current.clientHeight)
    }
  }, [isFocused])

  const handleBlur = (e: React.FocusEvent<HTMLTextAreaElement>) => {
    props.onBlur?.(e)
  }

  const handleFocus = (e: React.FocusEvent<HTMLTextAreaElement>) => {
    props.onFocus?.(e)
  }

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    props.onChange?.(e)
  }

  return (
    <Flex flex="1" h="100%" pos="relative" w="100%">
      <Box
        bg="transparent"
        borderRadius={['16px', null, null, '30px']}
        minH="25dvh"
        onClick={(e) => e.currentTarget.querySelector('textarea')?.focus()}
        overflow="hidden"
        p={['16px', null, null, '40px']}
        style={{ height: boxHeight + 32 }}
        transition="height 0.3s ease-in-out"
        w="100%"
      >
        <Input
          as="textarea"
          bg="$containerBackground"
          border="none"
          borderRadius={['16px', null, null, '30px']}
          color="$text"
          h="100%"
          left="0"
          onBlur={handleBlur}
          onChange={handleChange}
          onFocus={handleFocus}
          p={['16px', null, null, '40px']}
          pos="absolute"
          resize="none"
          top="0"
          typography="braille"
          value={props.value}
          w="100%"
          {...props}
        />
        {(!props.value || (props.value as string).length === 0) && (
          <Text
            ref={placeholderRef}
            color="$text"
            display="inline-block"
            h="fit-content"
            lineBreak="anywhere"
            opacity={0.5}
            typography="braille"
            whiteSpace="pre-line"
          >
            {isFocused ? focusPlaceholder : blurPlaceholder}
          </Text>
        )}
      </Box>
    </Flex>
  )
}
