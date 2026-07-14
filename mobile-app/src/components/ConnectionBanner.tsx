import React, { useEffect, useRef } from 'react';
import { View, Text, StyleSheet, Animated } from 'react-native';

interface ConnectionBannerProps {
  isOnline: boolean;
  showBanner: boolean;
}

export default function ConnectionBanner({ isOnline, showBanner }: ConnectionBannerProps) {
  const opacity = useRef(new Animated.Value(0)).current;

  useEffect(() => {
    if (showBanner) {
      Animated.sequence([
        Animated.timing(opacity, {
          toValue: 1,
          duration: 300,
          useNativeDriver: true,
        }),
        Animated.delay(3000),
        Animated.timing(opacity, {
          toValue: 0,
          duration: 300,
          useNativeDriver: true,
        }),
      ]).start();
    }
  }, [showBanner, opacity]);

  if (!showBanner) return null;

  return (
    <Animated.View
      style={[
        styles.container,
        isOnline ? styles.online : styles.offline,
        { opacity },
      ]}
    >
      <View style={[styles.dot, isOnline ? styles.dotOnline : styles.dotOffline]} />
      <Text style={styles.text}>
        {isOnline ? 'Back online' : 'You are offline'}
      </Text>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 8,
    paddingHorizontal: 16,
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    zIndex: 100,
  },
  online: {
    backgroundColor: 'rgba(34, 197, 94, 0.9)',
  },
  offline: {
    backgroundColor: 'rgba(239, 68, 68, 0.9)',
  },
  dot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 8,
  },
  dotOnline: {
    backgroundColor: '#ffffff',
  },
  dotOffline: {
    backgroundColor: '#ffffff',
  },
  text: {
    color: '#ffffff',
    fontSize: 14,
    fontWeight: '600',
  },
});
