import * as React from 'react'

import { View, Text } from 'react-native'
import { indyVdr, registerIndyVdr } from 'indy-vdr-shared'
import { indyVdrReactNative } from 'indy-vdr-react-native'

export default function App() {
  // registerIndyVdr({ vdr: indyVdrReactNative })
  return (
    <View style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
      <Text>{indyVdr.version()}</Text>
    </View>
  )
}
