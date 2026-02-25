import type { Params } from '../../server/request/params'
import type { ReadonlyURLSearchParams } from './readonly-url-search-params'
import { workUnitAsyncStorage } from '../../server/app-render/work-unit-async-storage.external'
import { workAsyncStorage } from '../../server/app-render/work-async-storage.external'
import {
  createExhaustiveParamsProxy,
  createExhaustiveURLSearchParamsProxy,
} from '../../server/app-render/instant-validation/sample-request-data'

export function instrumentParamsForValidation<TPArams extends Params>(
  underlyingParams: TPArams
): TPArams {
  const workStore = workAsyncStorage.getStore()
  const workUnitStore = workUnitAsyncStorage.getStore()
  if (workStore && workUnitStore) {
    switch (workUnitStore.type) {
      case 'validation-client': {
        if (workUnitStore.validationSamples) {
          const declaredKeys = new Set(
            Object.keys(workUnitStore.validationSamples.params ?? {})
          )
          return createExhaustiveParamsProxy(
            underlyingParams,
            declaredKeys,
            workStore.route
          )
        }
        break
      }
      case 'prerender-runtime':
      case 'prerender-client':
      case 'prerender-legacy':
      case 'prerender-ppr':
      case 'prerender':
      case 'cache':
      case 'request':
      case 'private-cache':
      case 'unstable-cache':
        break
      default:
        workUnitStore satisfies never
    }
  }
  return underlyingParams
}

export function throwForIncompleteParamsSampleInValidation(
  expression: string
): void {
  const workStore = workAsyncStorage.getStore()
  const workUnitStore = workUnitAsyncStorage.getStore()
  if (workStore && workUnitStore) {
    switch (workUnitStore.type) {
      case 'validation-client': {
        if (workUnitStore.validationSamples) {
          const fallbackParams = workUnitStore.fallbackRouteParams
          if (fallbackParams && fallbackParams.size > 0) {
            const missingParams = Array.from(fallbackParams.keys())
            const error = new Error(
              `Route "${workStore.route}" called ${expression} but param${missingParams.length > 1 ? 's' : ''} ${missingParams.map((p) => `"${p}"`).join(', ')} ${missingParams.length > 1 ? 'are' : 'is'} not defined in the \`samples\` of \`unstable_instant\`. ` +
                `${expression} requires all route params to be provided.`
            )
            ;(error as any).digest =
              'INSTANT_VALIDATION_EXHAUSTIVE_SAMPLES_ERROR'
            throw error
          }
        }
        break
      }
      case 'prerender-runtime':
      case 'prerender-client':
      case 'prerender-legacy':
      case 'prerender-ppr':
      case 'prerender':
      case 'cache':
      case 'request':
      case 'private-cache':
      case 'unstable-cache':
        break
      default:
        workUnitStore satisfies never
    }
  }
}

export function instrumentReadonlySearchParamsForValidation(
  underlyingSearchParams: ReadonlyURLSearchParams
): ReadonlyURLSearchParams {
  const workStore = workAsyncStorage.getStore()
  const workUnitStore = workUnitAsyncStorage.getStore()
  if (workStore && workUnitStore) {
    switch (workUnitStore.type) {
      case 'validation-client': {
        if (workUnitStore.validationSamples) {
          const declaredKeys = new Set(
            Object.keys(workUnitStore.validationSamples.searchParams ?? {})
          )
          return createExhaustiveURLSearchParamsProxy(
            underlyingSearchParams,
            declaredKeys,
            workStore.route
          )
        }
        break
      }
      case 'prerender-runtime':
      case 'prerender-client':
      case 'prerender-legacy':
      case 'prerender-ppr':
      case 'prerender':
      case 'cache':
      case 'request':
      case 'private-cache':
      case 'unstable-cache':
        break
      default:
        workUnitStore satisfies never
    }
  }
  return underlyingSearchParams
}
