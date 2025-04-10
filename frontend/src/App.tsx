import { useState } from 'react'
import { ChakraProvider, Box, Container, Heading, Text, VStack, Input, Button, Textarea, useToast, Spinner, Card, CardBody, Badge, Link, HStack, Tag, TagLabel, TagLeftIcon, Alert, AlertIcon, AlertTitle, AlertDescription, FormControl, FormLabel, SimpleGrid } from '@chakra-ui/react'
import { SearchIcon, StarIcon, DownloadIcon, TimeIcon, ExternalLinkIcon } from '@chakra-ui/icons'
import axios from 'axios'

interface CrateInfo {
  name: string
  description: string
  version: string
  downloads: number
  last_updated: string
  score: number
  repository?: string
  documentation?: string
  keywords: string[]
  url: string
}

interface RecommendationResponse {
  crates: CrateInfo[]
  explanation: string
}

function App() {
  const [query, setQuery] = useState('')
  const [context, setContext] = useState('')
  const [loading, setLoading] = useState(false)
  const [recommendations, setRecommendations] = useState<CrateInfo[]>([])
  const [error, setError] = useState<string | null>(null)
  const toast = useToast()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;

    setLoading(true);
    setError(null);
    try {
      const response = await fetch('http://127.0.0.1:3000/api/recommend', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query,
          context: '',
        }),
      });

      const data: RecommendationResponse = await response.json();
      
      if (data.crates.length === 0 && data.explanation) {
        setError(data.explanation);
        setRecommendations([]);
      } else {
        setRecommendations(data.crates);
        setError(null);
      }
    } catch (err) {
      setError('无法连接到服务器。请检查后端服务是否正在运行。');
      setRecommendations([]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <ChakraProvider>
      <Box minH="100vh" bg="gray.50" py={10}>
        <Container maxW="container.xl">
          <VStack spacing={8} align="stretch">
            <Heading as="h1" size="2xl" textAlign="center" color="blue.600">
              Rust Crate 智能推荐
            </Heading>
            <Text textAlign="center" fontSize="lg" color="gray.600">
              输入您的需求，我们将为您推荐最合适的 Rust 库
            </Text>

            <form onSubmit={handleSubmit}>
              <VStack spacing={4}>
                <FormControl>
                  <FormLabel>描述你的需求</FormLabel>
                  <Textarea
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder="例如：我需要一个处理 JSON 数据的库，要求性能好，使用简单"
                    size="lg"
                    rows={4}
                  />
                </FormControl>
                <Button
                  type="submit"
                  colorScheme="blue"
                  size="lg"
                  width="full"
                  isLoading={loading}
                >
                  获取推荐
                </Button>
              </VStack>
            </form>

            {error && (
              <Alert status="error" variant="subtle" flexDirection="column" alignItems="center" justifyContent="center" textAlign="center" height="200px">
                <AlertIcon boxSize="40px" mr={0} />
                <AlertTitle mt={4} mb={1} fontSize="lg">
                  出错了
                </AlertTitle>
                <AlertDescription maxWidth="sm">
                  {error}
                </AlertDescription>
              </Alert>
            )}

            {recommendations.length > 0 && (
              <Box>
                <Heading as="h2" size="lg" mb={4}>
                  推荐结果
                </Heading>
                <SimpleGrid columns={{ base: 1, md: 2, lg: 3 }} spacing={6}>
                  {recommendations.map((crate) => (
                    <Box
                      key={crate.name}
                      p={5}
                      shadow="md"
                      borderWidth="1px"
                      borderRadius="lg"
                    >
                      <VStack align="stretch" spacing={3}>
                        <Heading size="md">{crate.name}</Heading>
                        <Text>{crate.description}</Text>
                        <Badge colorScheme={crate.score > 0.7 ? "green" : crate.score > 0.4 ? "yellow" : "red"}>
                          匹配度: {(crate.score * 100).toFixed(0)}%
                        </Badge>
                        <Link href={crate.url} isExternal color="blue.500">
                          查看详情 →
                        </Link>
                      </VStack>
                    </Box>
                  ))}
                </SimpleGrid>
              </Box>
            )}
          </VStack>
        </Container>
      </Box>
    </ChakraProvider>
  )
}

export default App
