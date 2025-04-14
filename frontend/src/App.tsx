import { useState } from 'react'
import {
  ChakraProvider,
  Box,
  VStack,
  Heading,
  Text,
  Input,
  Button,
  FormControl,
  FormLabel,
  Textarea,
  useToast,
  Container,
  List,
  ListItem,
  Link,
  Badge,
  Divider,
  Alert,
  AlertIcon,
  AlertTitle,
  AlertDescription,
} from '@chakra-ui/react'

interface CrateInfo {
  name: string;
  description: string;
  version: string;
  downloads: number;
  last_updated: string;
  score: number;
  repository?: string;
  documentation?: string;
  keywords: string[];
}

interface RecommendationResponse {
  crates: CrateInfo[];
  explanation: string;
}

function App() {
  const [query, setQuery] = useState('');
  const [context, setContext] = useState('');
  const [loading, setLoading] = useState(false);
  const [recommendations, setRecommendations] = useState<CrateInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const toast = useToast();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim()) {
      toast({
        title: '错误',
        description: '请输入您的需求描述',
        status: 'error',
        duration: 3000,
        isClosable: true,
      });
      return;
    }

    setLoading(true);
    setError(null);
    setRecommendations([]);

    try {
      console.log('Sending request to backend...', { query, context });
      const response = await fetch('http://localhost:3000/api/recommend', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query,
          context: context || undefined,
        }),
      });

      console.log('Response status:', response.status);
      const data = await response.json();
      console.log('Response data:', data);

      if (!response.ok) {
        throw new Error(data.explanation || `请求失败: ${response.status}`);
      }

      if (data.crates && Array.isArray(data.crates)) {
        if (data.crates.length === 0) {
          setError('未找到匹配的 crates。请尝试使用不同的关键词或提供更多上下文信息。');
        } else {
          setRecommendations(data.crates);
          setError(null);
        }
      } else {
        throw new Error('返回数据格式不正确');
      }
    } catch (err) {
      console.error('Error:', err);
      const errorMessage = err instanceof Error ? err.message : '发生未知错误';
      setError(errorMessage);
      toast({
        title: '错误',
        description: errorMessage,
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <ChakraProvider>
      <Container maxW="container.xl" py={8}>
        <VStack spacing={8} align="stretch">
          <Heading as="h1" size="xl" textAlign="center">
            Rust Crate 推荐系统
          </Heading>
          
          <Box as="form" onSubmit={handleSubmit} p={6} borderWidth={1} borderRadius="lg">
            <VStack spacing={4}>
              <FormControl isRequired>
                <FormLabel>您的需求描述</FormLabel>
                <Textarea
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="请描述您需要什么样的 crate，例如：'需要一个用于处理 HTTP 请求的库'"
                  size="lg"
                />
              </FormControl>

              <FormControl>
                <FormLabel>上下文信息（可选）</FormLabel>
                <Textarea
                  value={context}
                  onChange={(e) => setContext(e.target.value)}
                  placeholder="提供更多上下文信息，例如：'这是一个 Web 服务器项目，需要处理并发请求'"
                  size="lg"
                />
              </FormControl>

              <Button
                type="submit"
                colorScheme="blue"
                size="lg"
                width="full"
                isLoading={loading}
                loadingText="正在搜索..."
              >
                获取推荐
              </Button>
            </VStack>
          </Box>

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
              <List spacing={4}>
                {recommendations.map((crate) => (
                  <ListItem
                    key={crate.name}
                    p={4}
                    borderWidth={1}
                    borderRadius="md"
                    _hover={{ bg: 'gray.50' }}
                  >
                    <VStack align="stretch" spacing={2}>
                      <Box display="flex" justifyContent="space-between" alignItems="center">
                        <Heading as="h3" size="md">
                          {crate.name}
                        </Heading>
                        <Badge colorScheme="green" fontSize="sm">
                          评分: {crate.score.toFixed(2)}
                        </Badge>
                      </Box>
                      
                      <Text>{crate.description}</Text>
                      
                      <Box display="flex" gap={4} fontSize="sm" color="gray.600">
                        <Text>版本: {crate.version}</Text>
                        <Text>下载量: {crate.downloads.toLocaleString()}</Text>
                        <Text>最后更新: {new Date(crate.last_updated).toLocaleDateString()}</Text>
                      </Box>

                      {crate.keywords && crate.keywords.length > 0 && (
                        <Box>
                          <Text fontSize="sm" fontWeight="bold" mb={1}>关键词：</Text>
                          <Box display="flex" gap={2} flexWrap="wrap">
                            {crate.keywords.map((keyword) => (
                              <Badge key={keyword} colorScheme="blue">
                                {keyword}
                              </Badge>
                            ))}
                          </Box>
                        </Box>
                      )}

                      <Box display="flex" gap={4}>
                        {crate.repository && (
                          <Link href={crate.repository} isExternal color="blue.500">
                            仓库地址
                          </Link>
                        )}
                        {crate.documentation && (
                          <Link href={crate.documentation} isExternal color="blue.500">
                            文档
                          </Link>
                        )}
                      </Box>
                    </VStack>
                  </ListItem>
                ))}
              </List>
            </Box>
          )}
        </VStack>
      </Container>
    </ChakraProvider>
  );
}

export default App;
