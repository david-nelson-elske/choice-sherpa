<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# Can you give an in depth guide on property based testing.  I am interest in developing invarriant featyres for my project that I can build specs for and test against.

Now I have comprehensive research on property-based testing. Let me create an in-depth guide focusing on developing invariant features and building testable specifications.

# Property-Based Testing: An In-Depth Guide to Invariants and Specifications

Property-based testing represents a paradigm shift from traditional example-based testing. Rather than writing individual test cases with specific inputs and expected outputs, you define **properties**—universal truths about your code that should hold for all valid inputs. The testing framework then generates diverse test scenarios automatically, helping uncover edge cases that manual testing often misses.[^1][^2]

## Understanding the Core Concepts

### What Are Properties?

Properties are logical assertions about your code's behavior that remain true regardless of the specific input values. Think of them as formal specifications expressed as executable code. Instead of asking "does my function return 5 when given 3 and 2?", you ask "does my function always satisfy this mathematical relationship for any valid inputs?"[^3][^1]

The power of property-based testing lies in **automated test case generation** and **systematic exploration** of your input space. When a property fails, frameworks employ **shrinking**—automatically reducing the failing test case to its simplest form, making debugging significantly easier.[^4][^5][^6]

### The Three Pillars

Property-based testing rests on three fundamental concepts:[^1]

**Generators** produce random or systematically varied test data within defined parameters. Good generators are fast, deterministic when seeded, and provide comprehensive coverage of the input space. They're the engine that explores your code's behavior across thousands of scenarios.

**Properties** are the specifications themselves—the invariants, relationships, and behavioral contracts your system must uphold. These form the heart of your testing strategy and directly encode your requirements.

**Shrinking** automatically minimizes failing test cases. When a property violation is discovered with complex inputs, the framework reduces them to the smallest example that still demonstrates the failure. This dramatically accelerates debugging by eliminating irrelevant complexity.[^5][^7][^8]

## Discovering Properties: Seven Patterns

Finding meaningful properties represents the most challenging aspect of property-based testing. Here are seven proven patterns for discovering testable properties in your systems:[^2][^9][^1]

### 1. Different Paths, Same Destination (Commutativity)

Some operations can be performed in any order and yield identical results. This pattern captures **commutative properties**.[^9]

**Example invariants:**

- Adding 1 then adding 2 equals adding 2 then adding 1
- Applying database migrations in parallel should yield the same final schema
- JSON serialization followed by parsing should equal direct object comparison

In mathematical notation, if operation X followed by operation Y produces the same result as Y followed by X, you have a commutative property.[^9]

### 2. There and Back Again (Invertibility)

**Round-trip properties** verify that paired operations are true inverses. This pattern is ubiquitous in software systems.[^10][^11][^9]

**Example invariants:**

- Encode then decode returns the original data
- Serialize then deserialize preserves all information
- Compress then decompress yields identical content
- Write to database then read returns same object
- `setProperty` then `getProperty` retrieves the set value

These properties are particularly valuable for testing parsers, codecs, serialization libraries, and storage systems.[^12][^13]

### 3. Some Things Never Change (Invariants Under Transformation)

Certain characteristics should remain constant despite operations performed on your data. These **structural invariants** define essential properties that survive transformations.[^10][^9]

**Example invariants:**

- List length remains unchanged after sorting
- Set membership is preserved through serialization
- Tree balance properties hold after insertions
- Total system energy remains constant (conservation laws)
- Sum of all account balances equals total supply in financial systems[^14]


### 4. The More Things Change, the More They Stay the Same (Idempotence)

**Idempotent operations** produce the same result whether applied once or multiple times. This property is critical for distributed systems, message processing, and database operations.[^15][^9]

**Example invariants:**

- Sorting a sorted list changes nothing
- Filtering duplicates twice equals filtering once
- Setting a value to X multiple times equals setting it once
- DELETE operations in REST APIs
- Creating a resource with the same ID repeatedly


### 5. Solve a Smaller Problem First (Structural Induction)

If a property holds for smaller parts of a structure, it often holds for larger structures built from those parts. This pattern leverages **recursive properties**.[^9]

**Example invariants:**

- If two sorted sublists maintain order, their merge is sorted
- Tree invariants in subtrees imply tree invariants in parent
- Correctness of components implies correctness of composition


### 6. Hard to Prove, Easy to Verify

Some problems are computationally expensive to solve but cheap to verify. Testing can focus on verification rather than implementation.[^9]

**Example invariants:**

- Sorting algorithm produces ordered output (checking order is O(n), sorting is O(n log n))
- Sudoku solution satisfies constraints (checking is faster than solving)
- Database query results satisfy WHERE clause predicates
- Optimization results meet constraints


### 7. Test Oracle (Reference Implementation)

Compare your optimized or complex implementation against a simpler, obviously correct reference implementation.[^13][^16][^10]

**Example invariants:**

- Optimized algorithm matches naive implementation
- Custom data structure behaves like standard library equivalent
- Fast approximation is within epsilon of precise calculation

This pattern is particularly effective when you have an existing correct implementation and want to verify a rewrite, optimization, or port.[^17][^18]

## Metamorphic Testing: Transformations with Known Outcomes

**Metamorphic testing** extends property-based testing by defining transformations where you know how the output should change relative to input changes. You don't need to know the exact output—only the relationship between outputs for related inputs.[^19][^10]

**Example metamorphic properties:**

- Searching for hotels in Sydney returns N results; filtering by price should return a subset of those N results[^19]
- Doubling audio volume shouldn't change speech-to-text transcription[^10]
- Rotating an image shouldn't change object detection confidence scores
- Linearly transforming inputs to an SVM shouldn't affect classification[^10]

Metamorphic testing is especially powerful for testing scientific algorithms, machine learning models, and systems where precise outputs are difficult to predict.[^20][^21][^19]

## Building Testable Specifications: A Practical Workflow

### Phase 1: Requirements Analysis and Property Extraction

Start by examining your requirements and extracting testable properties. For each function or system behavior, ask:[^22][^23][^2]

1. **What should always be true?** (invariants)
2. **What relationships exist between inputs and outputs?** (preconditions/postconditions)
3. **What operations should be reversible?** (round-trip properties)
4. **What operations can be reordered?** (commutativity)
5. **What operations are idempotent?** (stability under repetition)

Document these properties explicitly. They form your **executable specification**.[^23][^22]

### Phase 2: Define Your Domain Model

Property-based testing requires careful consideration of your input space. Define generators that produce:

- **Valid inputs**: Data that meets your function's preconditions
- **Boundary cases**: Edge values near limits
- **Representative distributions**: Inputs that mirror real-world usage
- **Problematic patterns**: Known difficult cases

For complex domain objects, compose generators from simpler primitives. Most frameworks provide built-in generators for common types (integers, strings, lists), which you combine to create domain-specific generators.[^24][^25][^13]

### Phase 3: Write Properties as Tests

Express each property as an executable test. The general pattern is:[^26][^27][^12]

```
property_name:
  for all (valid inputs x, y, z...):
    precondition(x, y, z) implies property_holds(x, y, z)
```

Use your framework's API to wire generators to property functions. Most frameworks follow this structure:

- **Hypothesis (Python)**: `@given` decorator with strategy arguments[^28][^12][^26]
- **fast-check (JavaScript/TypeScript)**: `fc.assert(fc.property(generators..., testFn))`[^29][^30][^13]
- **FsCheck (F\#/.NET)**: Property computations with generators[^4][^5]
- **jqwik (Java)**: `@Property` annotation with `@ForAll` parameters[^31][^32]
- **QuickCheck (Haskell)**: The original, using `forAll` and generators[^17][^26]
- **gopter (Go)**: Property definitions with generator combinators[^1]


### Phase 4: Implement Custom Generators

As your domain grows more complex, you'll need **custom generators**. Good generators balance several concerns:[^25][^33][^24]

- **Coverage**: Generate diverse inputs that exercise different code paths
- **Validity**: Produce only legal inputs that satisfy domain constraints
- **Performance**: Generate quickly to enable running thousands of test cases
- **Shrinking**: Support automatic minimization when failures occur

Most frameworks provide combinators for building complex generators from simple ones. Common patterns include:[^13][^29][^24]

- **Filtering**: Apply predicates to restrict generated values
- **Mapping**: Transform generated values
- **Binding/chaining**: Use one generated value to constrain another
- **Frequency/weighting**: Control distribution of generated values[^17]
- **Recursive generation**: Build tree structures and nested data


### Phase 5: Validate Your Generators

Generators themselves can have bugs. Validate them by:[^17]

- **Collecting statistics**: Many frameworks let you classify and report on generated test data[^18][^34]
- **Manual inspection**: Sample generated values and verify they match expectations
- **Coverage analysis**: Check that generators explore your input space adequately
- **Generator properties**: Write properties about your generators (they should produce valid data, diverse samples, etc.)


### Phase 6: Configure Test Execution

Property-based tests require configuration:[^35][^27]

- **Number of test cases**: 100-10,000+ depending on complexity and performance requirements
- **Random seed**: For reproducibility when failures occur
- **Shrinking parameters**: Control minimization aggressiveness
- **Timeout limits**: Prevent infinite loops in generators or properties
- **Examples**: Hard-code specific test cases to always run[^36]


### Phase 7: Interpret Failures and Debug

When a property fails, the framework reports:[^6][^37][^12]

- **The failing property**: Which assertion was violated
- **Minimal counterexample**: Smallest inputs that trigger failure (after shrinking)
- **Seed value**: For reproducing the exact test sequence
- **Shrinking trace**: Sometimes includes the path from original to minimal failure

Use the minimal counterexample to understand the root cause. The seed allows reproducing failures deterministically for debugging.[^7][^38][^32][^39][^5]

## Testing Stateful Systems: Model-Based Properties

Many systems are **stateful**—their behavior depends on a sequence of operations. Property-based testing extends naturally to these scenarios through **model-based testing**.[^40][^41][^38][^42][^18]

### The State Machine Approach

Define a simplified **model** that captures expected behavior alongside your real implementation:[^42][^40][^18]

1. **Model state**: Abstract representation of system state
2. **Commands**: Operations that modify both model and system
3. **Preconditions**: When each command is valid
4. **Postconditions**: What should be true after executing a command
5. **Invariants**: Properties that should always hold

The testing framework generates random **command sequences**, executing them against both the model and the system under test. After each command, it verifies:[^38][^40][^18]

- Model and system reach equivalent states
- Postconditions are satisfied
- Invariants are preserved

This approach is remarkably powerful for testing data structures, databases, caches, file systems, and protocols.[^34][^18][^42]

### Concurrent Testing and Race Conditions

Property-based state machine testing extends to concurrent scenarios. The framework:[^43][^18]

1. Generates a sequence of commands
2. Runs some sequentially to establish initial state
3. Executes others in parallel across multiple threads
4. Verifies the final state is consistent with some sequential interleaving (**linearizability**)[^18]

This technique excels at uncovering race conditions, deadlocks, and atomicity violations that are notoriously difficult to find through traditional testing.[^44][^43]

## Integration Testing with Contract-Tested Fakes

**Fakes** are test doubles that implement realistic behavior, often using in-memory data structures to simulate external systems like databases or APIs. Unlike mocks or stubs, fakes maintain internal state and execute real logic.[^45][^46][^34]

The challenge with fakes is ensuring they faithfully represent the real system. **Contract testing** addresses this by writing properties that both the fake and the real implementation must satisfy:[^46][^45][^18]

1. Define the **contract** as a set of properties
2. Test both real and fake implementations against these properties
3. Use the fake in tests with confidence it behaves like the real system

This approach provides:

- **Faster tests**: Fakes eliminate network calls and expensive I/O
- **Deterministic behavior**: No flaky tests from external dependencies
- **Parallel execution**: Tests don't interfere with shared resources
- **Documented contracts**: Properties explicitly capture behavioral expectations[^45][^46]


## Advanced Techniques and Best Practices

### Combining Property-Based and Example-Based Tests

Property-based testing complements rather than replaces example-based unit tests. Use both:[^47][^2][^44]

- **Examples**: Document specific important behaviors, capture known edge cases, serve as regression tests for discovered bugs
- **Properties**: Provide broad coverage, uncover unexpected failures, specify invariants[^44][^47]

Many developers start by converting existing unit tests into properties, gradually building intuition for property-based thinking.[^2][^1]

### Performance Considerations

Property-based tests are slower than single example tests—they run hundreds or thousands of cases. Optimize by:[^30]

- **Tiering**: Run quick property tests in development, exhaustive suites in CI[^39][^48]
- **Targeted generation**: Constrain generators to relevant input ranges
- **Parallel execution**: Run independent properties concurrently
- **Smart shrinking**: Modern frameworks use integrated shrinking strategies that minimize overhead[^8][^17]


### CI Integration and Regression Testing

Property-based tests work well in continuous integration when properly configured:[^48][^39]

- **Use fixed seeds** during debugging to reproduce specific failures
- **Cache failing examples** to detect regressions quickly
- **Report coverage statistics** from generated test data
- **Set reasonable timeouts** to prevent hanging builds

Some teams run property tests with fewer iterations on every commit, then exhaustive runs nightly or weekly.[^49][^39]

### Testing AI and ML Systems

Property-based testing adapts well to AI systems despite their non-deterministic nature:[^50][^21][^51][^20]

- **Behavioral properties**: Outputs maintain invariants (classification confidence sums to 1.0)
- **Metamorphic properties**: Known input transformations yield predictable output changes[^21][^10]
- **Distribution properties**: Outputs follow expected statistical distributions[^21]
- **Robustness properties**: Perturbations within bounds don't drastically change predictions[^51]

Testing AI systems focuses on **properties of distributions** rather than exact outputs.[^51][^21]

## Language-Specific Tooling

Property-based testing libraries exist for virtually all modern languages:[^52][^26][^4]

- **Python**: Hypothesis (mature, widely adopted, excellent shrinking)[^27][^12][^28][^26]
- **JavaScript/TypeScript**: fast-check (QuickCheck-inspired, integrates with Jest/Mocha)[^53][^36][^29][^30][^13]
- **Haskell**: QuickCheck (the original, gold standard)[^54][^26][^17]
- **Erlang/Elixir**: PropEr (designed for testing distributed systems)[^40][^42]
- **F\#/.NET**: FsCheck (functional-first, works with NUnit/xUnit)[^5][^4]
- **Java/Kotlin**: jqwik (JUnit 5 integration)[^55][^32][^31]
- **Go**: gopter (functional generators, shrinking support)[^1]
- **Scala**: Scalacheck (functional, state machine testing)[^38]
- **Rust**: proptest, quickcheck-rs (ownership-aware generators)
- **Solidity**: Foundry (invariant testing for smart contracts)[^14]

Choose based on ecosystem maturity, documentation quality, shrinking capabilities, and community support.[^54][^17]

## Common Pitfalls and How to Avoid Them

### Weak Properties

Properties that are too general don't catch bugs. "The function doesn't crash" is a property, but not a very useful one. Strengthen properties by:[^2]

- Specifying precise relationships between inputs and outputs
- Combining multiple properties for comprehensive coverage
- Using metamorphic relations when exact outputs are unknown


### Biased Generators

Generators that don't explore the full input space miss bugs. Ensure generators:[^1][^17]

- Cover boundary values (empty collections, zero, negative numbers, maximum values)
- Produce edge cases with reasonable probability
- Don't over-constrain inputs unless domain requires it


### Insufficient Shrinking

If your framework or custom generators don't shrink well, failures present as complex, difficult-to-debug examples. Invest in good shrinking:[^5][^17]

- Use framework primitives that provide integrated shrinking
- Test your custom generators' shrinking behavior
- Consider libraries with automatic shrinking (Hypothesis, Hedgehog)[^8][^54][^17]


### Confusing Properties with Implementation

Properties should specify **what** behavior is required, not **how** it's implemented. Test outputs and observable side effects, not internal implementation details. This keeps tests maintainable as code evolves.[^3][^46]

### Over-reliance on Property Tests

Property-based testing doesn't replace all other testing. It's one tool in your toolbox. Combine with:[^56][^49]

- Unit tests for specific scenarios
- Integration tests for system interactions
- Performance tests for scalability
- Security tests for vulnerabilities
- User acceptance tests for business requirements


## Getting Started: A Practical Roadmap

1. **Choose one function** with clear invariants (sorting, encoding, arithmetic operations)
2. **Write one simple property** using your language's framework
3. **Run it with default settings** (usually 100 tests) and observe results
4. **Add a second property** for the same function using a different pattern
5. **Introduce a bug** deliberately and verify your properties catch it
6. **Experiment with custom generators** for domain-specific types
7. **Convert an existing unit test** into a property-based test
8. **Gradually expand** to more complex functions and stateful systems[^57][^27]

Start small, build intuition, and incrementally adopt property-based testing where it provides the most value.[^52][^56][^2]

## Conclusion

Property-based testing shifts your mindset from "does my code work for these examples?" to "what properties should my code always satisfy?". By defining invariants as executable specifications, you create tests that:[^22][^2]

- **Explore input spaces thoroughly**, finding edge cases you'd never think to write
- **Document behavior formally**, serving as living specifications
- **Provide minimal counterexamples**, accelerating debugging when failures occur
- **Scale efficiently**, testing complex systems with modest test code[^12][^52]

The investment in learning property-based testing pays dividends through higher confidence, fewer production bugs, and clearer specifications. As systems grow more complex—especially distributed systems, stateful services, and AI/ML applications—property-based testing becomes increasingly valuable for maintaining correctness at scale.[^23][^44][^2]
<span style="display:none">[^58][^59][^60][^61][^62][^63][^64][^65][^66][^67][^68][^69][^70][^71][^72][^73][^74][^75][^76][^77][^78][^79][^80][^81][^82][^83][^84][^85][^86][^87][^88][^89][^90]</span>

<div align="center">⁂</div>

[^1]: https://dzone.com/articles/property-based-testing-guide-go

[^2]: https://www.shadecoder.com/topics/what-is-property-based-testing-a-practical-guide-for-2025

[^3]: https://www.thesoftwarelounge.com/the-beginners-guide-to-property-based-testing/

[^4]: https://antithesis.com/resources/property_based_testing/

[^5]: https://www.codit.eu/blog/f-fscheck-shrinkers-for-domain-model-generators/

[^6]: https://zio.dev/reference/test/property-testing/shrinking

[^7]: https://kotest.io/docs/proptest/property-test-shrinking.html

[^8]: https://getcode.substack.com/p/property-based-testing-5-shrinking

[^9]: https://fsharpforfunandprofit.com/posts/property-based-testing-2/

[^10]: https://alanhdu.github.io/posts/2023-07-14-property-based-testing/

[^11]: https://en.wikipedia.org/wiki/Round-trip_format_conversion

[^12]: https://increment.com/testing/in-praise-of-property-based-testing/

[^13]: https://github.com/dubzzz/fast-check-examples

[^14]: https://rareskills.io/post/invariant-testing-solidity

[^15]: https://www.jot.fm/issues/issue_2020_02/article7.pdf

[^16]: https://www.richard-seidl.com/en/blog/test-oracle-reference

[^17]: https://discourse.haskell.org/t/the-sad-state-of-property-based-testing-libraries/9880

[^18]: https://github.com/stevana/property-based-testing-stateful-systems-tutorial

[^19]: https://en.wikipedia.org/wiki/Metamorphic_testing

[^20]: https://towardsdatascience.com/how-to-test-machine-learning-systems-d53623d32797/

[^21]: https://galileo.ai/blog/unit-testing-ai-systems-first-principles

[^22]: https://kiro.dev/blog/property-based-testing/

[^23]: https://www.richard-seidl.com/en/blog/propertybased-testing

[^24]: https://propertesting.com/book_custom_generators.html

[^25]: https://www.leadingagile.com/2018/04/step-by-step-toward-property-based-testing/

[^26]: https://joss.theoj.org/papers/10.21105/joss.01891.pdf

[^27]: https://semaphore.io/blog/property-based-testing-python-hypothesis-pytest

[^28]: https://www.youtube.com/watch?v=mkgd9iOiICc

[^29]: https://github.com/dubzzz/fast-check

[^30]: https://www.davideaversa.it/blog/property-based-testing-typescript-fast-check/

[^31]: https://blog.johanneslink.net/2018/03/29/jqwik-on-junit5/

[^32]: https://www.youtube.com/watch?v=dPhZIo27fYE

[^33]: https://elixirforum.com/t/defining-custom-generators-with-propcheck/6401

[^34]: https://stevana.github.io/the_sad_state_of_property-based_testing_libraries.html

[^35]: https://kotest.io/docs/proptest/property-based-testing.html

[^36]: https://jrsinclair.com/articles/2021/how-to-get-started-with-property-based-testing-in-javascript-with-fast-check/

[^37]: https://www.yld.io/blog/the-oracle-problem

[^38]: https://xebia.com/blog/stateful-testing-in-scala/

[^39]: https://blog.nelhage.com/post/property-testing-like-afl/

[^40]: https://hexdocs.pm/propcheck/PropCheck.StateM.ModelDSL.html

[^41]: https://giacomociti.github.io/2021/08/19/Model-based-testing-made-simplistic.html

[^42]: https://proper-testing.github.io/tutorials/PropEr_testing_of_finite_state_machines.html

[^43]: https://digitalcommons.unl.edu/cgi/viewcontent.cgi?article=1209\&context=cseconfwork

[^44]: https://www.softwaretestingmagazine.com/knowledge/how-to-master-property-based-testing-for-reliable-software/

[^45]: https://blog.ploeh.dk/2023/11/13/fakes-are-test-doubles-with-contracts/

[^46]: https://quii.gitbook.io/learn-go-with-tests/testing-fundamentals/working-without-mocks

[^47]: https://www.softwaretestingmagazine.com/videos/best-practice-for-property-based-testing/

[^48]: https://digital.ai/catalyst-blog/implementing-continuous-testing/

[^49]: https://news.ycombinator.com/item?id=39850087

[^50]: https://soft.vub.ac.be/Publications/2024/vub-tr-soft-24-04.pdf

[^51]: https://techcommunity.microsoft.com/blog/azure-ai-foundry-blog/testing-modern-ai-systems-from-rule-based-systems-to-deep-learning-and-large-lan/4429518

[^52]: https://www.infoq.com/news/2024/12/fuzzy-unit-testing/

[^53]: https://www.grusingh.com/post/property-based-testing-with-fast-check/

[^54]: https://seelengrab.github.io/articles/The properties of QuickCheck, Hedgehog and Hypothesis/

[^55]: https://www.youtube.com/watch?v=TWxI5FXAae0

[^56]: https://www.thecoder.cafe/p/property-based-testing

[^57]: https://andrewhead.info/assets/pdf/pbt-in-practice.pdf

[^58]: https://www.youtube.com/watch?v=LA-VbEDGmzI

[^59]: https://lemonidas.github.io/pdf/Foundational.pdf

[^60]: https://news.ycombinator.com/item?id=15795712

[^61]: https://stackoverflow.com/questions/72359149/when-to-choose-example-based-testing-and-property-based-for-stateful-testing

[^62]: https://software.imdea.org/~marron/papers/mod_inv_prop.pdf

[^63]: https://www.reddit.com/r/Python/comments/2zt1l1/hypothesis_is_an_advanced_quickcheck_style/

[^64]: https://www.hillelwayne.com/post/hypothesis-oracles/

[^65]: https://martinfowler.com/bliki/TestInvariant.html

[^66]: https://metacpan.org/pod/Sereal::Encoder

[^67]: https://www.mat.uniroma2.it/~liverani/Lavori/live0203.pdf

[^68]: https://blogs.oracle.com/javamagazine/java-json-serialization-jackson/

[^69]: https://www.susanpotter.net/talks/thinking-in-properties-testing/

[^70]: https://clubztutoring.com/ed-resources/math/invariant-definitions-examples-6-7-6/

[^71]: https://rauljordan.com/go-lessons-from-writing-a-serialization-library-for-ethereum/

[^72]: https://blog.ssanj.net/posts/2016-06-26-property-based-testing-patterns.html

[^73]: https://cs.uwaterloo.ca/~elena-g/papers/bgs-camera.pdf

[^74]: https://uhi.readthedocs.io/en/latest/serialization.html

[^75]: https://kiro.dev/docs/specs/correctness/

[^76]: https://github.com/leanovate/gopter/issues/22

[^77]: https://dev.to/mokkapps/property-based-testing-with-typescript-2ljj

[^78]: https://www.globalapptesting.com/blog/what-is-continuous-testing

[^79]: https://mokkapps.de/blog/property-based-testing-with-type-script

[^80]: https://dev.to/meeshkan/property-based-testing-for-javascript-developers-21b2

[^81]: https://www.cs.purdue.edu/homes/apm/foundationsBook/samples/fsm-chapter.pdf

[^82]: https://en.wikipedia.org/wiki/Property_testing

[^83]: https://cs.unibg.it/gargantini/didattica/swtestandver/restricted/appunti16_17/5_modelbased_testing/5_3_Chapter5_MBTBook.pdf

[^84]: https://www.wisdom.weizmann.ac.il/~oded/PDF/dana-tech.pdf

[^85]: https://www.hse.ru/mirror/pubs/share/206233457

[^86]: https://www.sciencedirect.com/science/article/pii/S0167642323000874

[^87]: https://cs.uwaterloo.ca/~eblais/assets/pdf/active_testing.pdf

[^88]: https://people.scs.carleton.ca/~jeanpier/COMP5104F06/5104-F06-L14.pdf

[^89]: https://www.linkedin.com/pulse/using-contract-tests-reliable-memory-fakes-shai-yallin-9pkaf

[^90]: https://www.cs.princeton.edu/courses/archive/spr04/cos598B/bib/Ron-tutorial.pdf

