cmake_minimum_required(VERSION 3.9.0)

set (PACKAGE_NAME "indyvdrreactnative")
set (LIB_NAME "indyvdr")
set (BUILD_DIR ${CMAKE_SOURCE_DIR}/build)

include_directories(
  ${CMAKE_SOURCE_DIR}/libs/${ANDROID_ABI}
  ../cpp
  ../cpp/include
  "${NODE_MODULES_DIR}/react-native/React"
  "${NODE_MODULES_DIR}/react-native/React/Base"
  "${NODE_MODULES_DIR}/react-native/ReactCommon/jsi"
)

add_library(
  ${PACKAGE_NAME}
  SHARED
  cpp-adapter.cpp
  ../cpp/HostObject.cpp
  ../cpp/turboModuleUtility.cpp
  ../cpp/indyVdr.cpp
)

set_target_properties(
  ${PACKAGE_NAME}
  PROPERTIES
  CXX_STANDARD 17
  CXX_EXTENSION OFF
  POSITION_INDEPENDENT_CODE ON
)

file (GLOB LIBRN_DIR  "${PREBUILT_DIR}/${ANDROID_ABI}")
file (GLOB LIBVDR_DIR "${CMAKE_SOURCE_DIR}/libs/${ANDROID_ABI}")

find_library(
  LOG_LIB
  log
)

find_library(
  REACT_NATIVE_JNI_LIB
  reactnativejni
  PATHS ${LIBRN_DIR}
  NO_CMAKE_FIND_ROOT_PATH
)

find_library(
  JSI_LIB
  jsi
  PATHS ${LIBRN_DIR}
  NO_CMAKE_FIND_ROOT_PATH
)

find_library(
  VDR_LIB
  indy_vdr
  paths ${LIBVDR_DIR}
  NO_CMAKE_FIND_ROOT_PATH
)

if (NOT VDR_LIB)
  message(FATAL_ERROR "Could not find VDR_LIB at: ${LIBVDR_DIR}")
endif()

target_link_libraries(
  ${PACKAGE_NAME}
  ${VDR_LIB}
  ${LOG_LIB}
  ${JSI_LIB}
  ${REACT_NATIVE_JNI_LIB}
  android
)
