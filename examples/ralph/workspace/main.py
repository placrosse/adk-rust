def main():
    while True:
        user_input = input("Enter operation (or 'quit' to exit): ")
        if user_input.lower() == 'quit':
            break
        try:
            result = eval(user_input)
            print(f'Result: {result}')
        except Exception as e:
            print(f'Error: {e}')

if __name__ == '__main__':
    main()