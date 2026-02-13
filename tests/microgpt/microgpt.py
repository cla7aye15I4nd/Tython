"""
The most atomic way to train and inference a GPT in pure, dependency-free Python.
This file is the complete algorithm.
Everything else is just efficiency.
@karpathy
"""

import math     # math.log, math.exp
import random   # random.seed, random.choices, random.gauss, random.shuffle

# Let there be Autograd, to recursively apply the chain rule through a computation graph
class Value:
    data: float
    grad: float
    _children: list['Value']
    _local_grads: list[float]

    def __init__(self, data: float, children: list['Value'] = [], local_grads: list[float] = []) -> None:
        self.data = data                # scalar value of this node calculated during forward pass
        self.grad = 0.0                 # derivative of the loss w.r.t. this node, calculated in backward pass
        self._children = children       # children of this node in the computation graph
        self._local_grads = local_grads # local derivative of this node w.r.t. its children

    def __add__(self, other: 'Value') -> 'Value':
        return Value(self.data + other.data, [self, other], [1.0, 1.0])

    def __mul__(self, other: 'Value') -> 'Value':
        return Value(self.data * other.data, [self, other], [other.data, self.data])

    def __pow__(self, other: 'Value') -> 'Value': return Value(self.data**other.data, [self, other], [other.data * self.data**(other.data-1)])
    def log(self) -> 'Value': return Value(math.log(self.data), [self], [1/self.data])
    def exp(self) -> 'Value': return Value(math.exp(self.data), [self], [math.exp(self.data)])
    def relu(self) -> 'Value': return Value(max(0, self.data), [self], [float(self.data > 0)])
    def __neg__(self) -> 'Value': return self * Value(-1.0)
    def __radd__(self, other: 'Value') -> 'Value': return self + other
    def __sub__(self, other: 'Value') -> 'Value': return self + (-other)
    def __rsub__(self, other: 'Value') -> 'Value': return other + (-self)
    def __rmul__(self, other: 'Value') -> 'Value': return self * other
    def __truediv__(self, other: 'Value') -> 'Value': return self * (other ** Value(-1.0))
    def __rtruediv__(self, other: 'Value') -> 'Value': return other * (self ** Value(-1.0))

    def backward(self) -> None:
        topo: list['Value'] = []
        visited: set['Value'] = set()
        def build_topo(v: 'Value') -> None:
            if v not in visited:
                visited.add(v)
                for child in v._children:
                    build_topo(child)
                topo.append(v)
        build_topo(self)
        self.grad = 1.0
        for v in reversed(topo):
            for child, local_grad in zip(v._children, v._local_grads):
                child.grad += local_grad * v.grad

# Define the model architecture: a stateless function mapping token sequence and parameters to logits over what comes next.
# Follow GPT-2, blessed among the GPTs, with minor differences: layernorm -> rmsnorm, no biases, GeLU -> ReLU
def linear(x: list[Value], w: list[list[Value]]) -> list[Value]:
    return [sum((wi * xi for wi, xi in zip(wo, x)), Value(0)) for wo in w]

def softmax(logits: list[Value]) -> list[Value]:
    max_val: float = max(val.data for val in logits)
    max_val_v: Value = Value(max_val)
    exps: list[Value] = [(val - max_val_v).exp() for val in logits]
    total: Value = sum(exps, Value(0))
    return [e / total for e in exps]

def rmsnorm(x: list[Value]) -> list[Value]:
    ms: Value = sum((xi * xi for xi in x), Value(0)) / Value(len(x))
    scale: Value = (ms + Value(1e-5)) ** Value(-0.5)
    return [xi * scale for xi in x]

if __name__ == '__main__':
    random.seed(42) # Let there be order among chaos

    # Let there be an input dataset `docs`: list[str] of documents (e.g. a dataset of names)
    docs: list[str] = [l.strip() for l in open('input.txt').read().strip().split('\n') if l.strip()]
    random.shuffle(docs)
    print(f"num docs: {len(docs)}")

    # Let there be a Tokenizer to translate strings to discrete symbols and back
    uchars: list[str] = sorted(set(''.join(docs))) # unique characters in the dataset become token ids 0..n-1
    BOS: int = len(uchars) # token id for the special Beginning of Sequence (BOS) token
    vocab_size: int = len(uchars) + 1 # total number of unique tokens, +1 is for BOS
    print(f"vocab size: {vocab_size}")

    # Initialize the parameters, to store the knowledge of the model.
    n_embd: int = 16     # embedding dimension
    n_head: int = 4      # number of attention heads
    n_layer: int = 1     # number of layers
    block_size: int = 16 # maximum sequence length
    head_dim: int = n_embd // n_head # dimension of each head
    def matrix(nout: int, nin: int, std: float = 0.08) -> list[list[Value]]:
        return [[Value(random.gauss(0, std)) for _ in range(nin)] for _ in range(nout)]
    state_dict: dict[str, list[list[Value]]] = {'wte': matrix(vocab_size, n_embd), 'wpe': matrix(block_size, n_embd), 'lm_head': matrix(vocab_size, n_embd)}
    for i in range(n_layer):
        state_dict[f'layer{i}.attn_wq'] = matrix(n_embd, n_embd)
        state_dict[f'layer{i}.attn_wk'] = matrix(n_embd, n_embd)
        state_dict[f'layer{i}.attn_wv'] = matrix(n_embd, n_embd)
        state_dict[f'layer{i}.attn_wo'] = matrix(n_embd, n_embd)
        state_dict[f'layer{i}.mlp_fc1'] = matrix(4 * n_embd, n_embd)
        state_dict[f'layer{i}.mlp_fc2'] = matrix(n_embd, 4 * n_embd)
    params: list[Value] = [p for mat in state_dict.values() for row in mat for p in row] # flatten params into a single list[Value]
    print(f"num params: {len(params)}")

    # Define the gpt function (references local state)
    def gpt(token_id: int, pos_id: int, keys: list[list[list[Value]]], values: list[list[list[Value]]]) -> list[Value]:
        tok_emb: list[Value] = state_dict['wte'][token_id] # token embedding
        pos_emb: list[Value] = state_dict['wpe'][pos_id] # position embedding
        x: list[Value] = [t + p for t, p in zip(tok_emb, pos_emb)] # joint token and position embedding
        x = rmsnorm(x)

        for li in range(n_layer):
            # 1) Multi-head attention block
            x_residual: list[Value] = x
            x = rmsnorm(x)
            q: list[Value] = linear(x, state_dict[f'layer{li}.attn_wq'])
            k: list[Value] = linear(x, state_dict[f'layer{li}.attn_wk'])
            v: list[Value] = linear(x, state_dict[f'layer{li}.attn_wv'])
            keys[li].append(k)
            values[li].append(v)
            x_attn: list[Value] = []
            for h in range(n_head):
                hs: int = h * head_dim
                q_h: list[Value] = q[hs:hs+head_dim]
                k_h: list[list[Value]] = [ki[hs:hs+head_dim] for ki in keys[li]]
                v_h: list[list[Value]] = [vi[hs:hs+head_dim] for vi in values[li]]
                attn_logits: list[Value] = [sum((q_h[j] * k_h[t][j] for j in range(head_dim)), Value(0)) / Value(head_dim**0.5) for t in range(len(k_h))]
                attn_weights: list[Value] = softmax(attn_logits)
                head_out: list[Value] = [sum((attn_weights[t] * v_h[t][j] for t in range(len(v_h))), Value(0)) for j in range(head_dim)]
                x_attn.extend(head_out)
            x = linear(x_attn, state_dict[f'layer{li}.attn_wo'])
            x = [a + b for a, b in zip(x, x_residual)]
            # 2) MLP block
            x_residual = x
            x = rmsnorm(x)
            x = linear(x, state_dict[f'layer{li}.mlp_fc1'])
            x = [xi.relu() for xi in x]
            x = linear(x, state_dict[f'layer{li}.mlp_fc2'])
            x = [a + b for a, b in zip(x, x_residual)]

        logits: list[Value] = linear(x, state_dict['lm_head'])
        return logits

    # Let there be Adam, the blessed optimizer and its buffers
    learning_rate: float = 0.01
    beta1: float = 0.85
    beta2: float = 0.99
    eps_adam: float = 1e-8
    m: list[float] = [0.0] * len(params) # first moment buffer
    v: list[float] = [0.0] * len(params) # second moment buffer

    # Repeat in sequence
    num_steps: int = 1000 # number of training steps
    for step in range(num_steps):

        # Take single document, tokenize it, surround it with BOS special token on both sides
        doc: str = docs[step % len(docs)]
        tokens: list[int] = [BOS] + [uchars.index(ch) for ch in doc] + [BOS]
        n: int = min(block_size, len(tokens) - 1)

        # Forward the token sequence through the model, building up the computation graph all the way to the loss.
        keys: list[list[list[Value]]] = [[] for _ in range(n_layer)]
        values: list[list[list[Value]]] = [[] for _ in range(n_layer)]
        losses: list[Value] = []
        for pos_id in range(n):
            token_id: int = tokens[pos_id]
            target_id: int = tokens[pos_id + 1]
            logits: list[Value] = gpt(token_id, pos_id, keys, values)
            probs: list[Value] = softmax(logits)
            loss_t: Value = -probs[target_id].log()
            losses.append(loss_t)
        loss: Value = Value(1.0 / n) * sum(losses, Value(0)) # final average loss over the document sequence. May yours be low.

        # Backward the loss, calculating the gradients with respect to all model parameters.
        loss.backward()

        # Adam optimizer update: update the model parameters based on the corresponding gradients.
        lr_t: float = learning_rate * (1 - step / num_steps) # linear learning rate decay
        for i, p in enumerate(params):
            m[i] = beta1 * m[i] + (1 - beta1) * p.grad
            v[i] = beta2 * v[i] + (1 - beta2) * p.grad ** 2
            m_hat: float = m[i] / (1 - beta1 ** (step + 1))
            v_hat: float = v[i] / (1 - beta2 ** (step + 1))
            p.data -= lr_t * m_hat / (v_hat ** 0.5 + eps_adam)
            p.grad = 0.0

        print(f"step {step+1:4d} / {num_steps:4d} | loss {loss.data:.4f}")

    # Inference: may the model babble back to us
    temperature: float = 0.5 # in (0, 1], control the "creativity" of generated text, low to high
    print("\n--- inference (new, hallucinated names) ---")
    for sample_idx in range(20):
        keys = [[] for _ in range(n_layer)]
        values = [[] for _ in range(n_layer)]
        token_id = BOS
        sample: list[str] = []
        for pos_id in range(block_size):
            logits = gpt(token_id, pos_id, keys, values)
            probs = softmax([l / Value(temperature) for l in logits])
            token_id = random.choices(range(vocab_size), weights=[p.data for p in probs])[0]
            if token_id == BOS:
                break
            sample.append(uchars[token_id])
        print(f"sample {sample_idx+1:2d}: {''.join(sample)}")
