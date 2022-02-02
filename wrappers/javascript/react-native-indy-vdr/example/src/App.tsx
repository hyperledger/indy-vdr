import * as React from 'react'

import { StyleSheet, View, Text } from 'react-native'

const App = () => (
  <View style={styles.container}>
    <Text>Hello, world!</Text>
  </View>
)

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
})

export default App
